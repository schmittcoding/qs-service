"""Assemble reviewed source-derived fragments into the distributable contract.

No network requests, no database access. Run from any directory with Python 3.11+.
--check requires openapi-spec-validator/jsonschema (requirements-docs.txt).
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
CONTRACT_DATE = "2026-10-16"
METHODS = {"get", "post", "put", "patch", "delete"}
FRAGMENTS = ("payments", "webhooks", "recharge-admin")
# Route files and the URL prefix actix mounts them under (main.rs + routes/mod.rs + routes/admin/mod.rs, API_VERSION=1).
ROUTE_SOURCES = {
    "ran-access-point/src/routes/payments/mod.rs": "/api/v1/payments",
    "ran-access-point/src/routes/admin/recharge/mod.rs": "/admin/v1/recharge",
}
SOURCE_FILES = [
    *ROUTE_SOURCES,
    "ran-access-point/src/routes/payments/paymongo_integration.rs",
    "ran-access-point/src/routes/mod.rs",
    "ran-access-point/src/routes/admin/mod.rs",
    "ran-access-point/src/services/jwt.rs",
    "ran-access-point/src/middleware/session/mod.rs",
    "ran-models/src/payments/mod.rs",
    "ran-models/src/payments/gcash/mod.rs",
    "ran-models/src/payments/wise/mod.rs",
    "ran-models/src/admin/mod.rs",
    "ran-models/src/lib.rs",
    "ran-db-core/src/webmanager/transaction/mod.rs",
    "db-scripts/webdb/Procedures/Administration/sp_GetWebTransactions.sql",
    "db-scripts/webdb/Triggers/trg_AddWebPoints.sql",
]


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf8", newline="\n")


def assemble():
    spec = {
        "openapi": "3.1.1",
        "info": {
            "title": "RAN Online GS Payments & Recharge API",
            "version": CONTRACT_DATE,
            "description": (
                "Source-derived contract for the gs-api player payment endpoints (/api/v1/payments), "
                "the PayPal and PayMongo webhooks, and the administrator recharge endpoints (/admin/v1/recharge). "
                "Documents current behavior, including known quirks (see errors.md). Players authenticate with an "
                "actix-session cookie; administrators with an HS256 bearer JWT. Request bodies are not strict: "
                "unknown JSON fields are ignored. Examples are fixtures, not production records. See README.md."
            ),
        },
        "servers": [
            {"url": "{api_origin}", "description": "gs-api origin (local dev binds APP_HOST:APP_PORT, e.g. http://127.0.0.1:5010)",
             "variables": {"api_origin": {"default": "http://127.0.0.1:5010"}}},
        ],
        "paths": {},
        "components": {},
        "x-contract-date": CONTRACT_DATE,
        "x-request-body-limit-bytes": 2097152,
        "x-client-request-timeout-seconds": 10,
        "x-source-authority": "actix-web handlers, Serde types, tiberius row mapping, and SQL; no inferred endpoints.",
    }
    for name in FRAGMENTS:
        fragment = json.loads((ROOT / "parts" / (name + ".openapi.json")).read_text(encoding="utf8"))
        for path, item in fragment["paths"].items():
            assert path not in spec["paths"], "Duplicate path: " + path
            spec["paths"][path] = item
        for section, entries in fragment.get("components", {}).items():
            target = spec["components"].setdefault(section, {})
            for key, value in entries.items():
                assert key not in target, "Duplicate component: " + section + "/" + key
                target[key] = value
    tags = []
    for item in spec["paths"].values():
        for method, operation in item.items():
            if method not in METHODS:
                continue
            for tag in operation.get("tags", []):
                if tag not in tags:
                    tags.append(tag)
            operation.setdefault("security", [])
            operation.setdefault("x-source-verified", True)
    spec["tags"] = [{"name": tag} for tag in tags]
    write_json(ROOT / "openapi.json", spec)
    return spec


def resolve(spec, value):
    if not isinstance(value, dict) or "$ref" not in value:
        return value
    pointer = value["$ref"]
    assert pointer.startswith("#/"), "Only bundled local references are supported"
    result = spec
    for token in pointer[2:].split("/"):
        result = result[token.replace("~1", "/").replace("~0", "~")]
    return resolve(spec, result)


def sample(spec, schema, depth=0):
    schema = resolve(spec, schema)
    if depth > 15:
        raise ValueError("Recursive schema needs an explicit example")
    if schema.get("examples"):
        return schema["examples"][0]
    if "example" in schema:
        return schema["example"]
    if "const" in schema:
        return schema["const"]
    if "enum" in schema:
        return schema["enum"][0]
    if "default" in schema:
        return schema["default"]
    for union in ("oneOf", "anyOf"):
        if union in schema:
            return sample(spec, next((s for s in schema[union] if s.get("type") != "null"), schema[union][0]), depth + 1)
    kind = schema.get("type")
    if isinstance(kind, list):
        kind = next((s for s in kind if s != "null"), "null")
    if kind == "object" or "properties" in schema:
        return {key: sample(spec, value, depth + 1) for key, value in schema.get("properties", {}).items() if key in schema.get("required", [])}
    if kind == "array":
        return [sample(spec, schema["items"], depth + 1) for _ in range(schema.get("minItems", 0))]
    if kind in ("integer", "number"):
        return max(1, schema.get("minimum", 1))
    if kind == "boolean":
        return False
    if kind == "null":
        return None
    return "EXAMPLE_VALUE"


def request_variants(spec, media):
    if "examples" in media:
        return [(name, resolve(spec, value)["value"]) for name, value in media["examples"].items()]
    if "example" in media:
        return [("example", media["example"])]
    return [("example", sample(spec, media.get("schema", {})))]


def make_postman(spec):
    groups = {}
    all_examples = []
    variables = {"base_url": "http://127.0.0.1:5010", "cookie_name": "YXGames", "session_cookie": "", "admin_jwt": ""}
    for path, item in spec["paths"].items():
        for method, operation in item.items():
            if method not in METHODS:
                continue
            parameters = [resolve(spec, p) for p in item.get("parameters", []) + operation.get("parameters", [])]
            media_type, media = None, None
            if "requestBody" in operation:
                body = resolve(spec, operation["requestBody"])
                media_type, media = next(iter(body["content"].items()))
            variants = request_variants(spec, media) if media else [("", None)]
            if media and not resolve(spec, operation["requestBody"]).get("required", False):
                variants.append(("no body", None))
            schemes = {name for requirement in operation.get("security", []) for name in requirement}
            for variant, value in variants:
                url_path = re.sub(r"\{([^}]+)\}", lambda m: "{{" + m[1] + "}}", path)
                for parameter in parameters:
                    if parameter["in"] == "path":
                        variables.setdefault(parameter["name"], str(parameter.get("example", sample(spec, parameter["schema"]))))
                query = [{"key": p["name"], "value": str(p.get("example", sample(spec, p["schema"]))), "disabled": not p.get("required", False),
                          "description": p.get("description", "")}
                         for p in parameters if p["in"] == "query"]
                headers = [{"key": "Accept", "value": "application/json"}]
                auth = {"type": "noauth"}
                if "AdminBearerJWT" in schemes:
                    auth = {"type": "bearer", "bearer": [{"key": "token", "type": "string", "value": "{{admin_jwt}}"}]}
                if "UserSessionCookie" in schemes:
                    headers.append({"key": "Cookie", "value": "{{cookie_name}}={{session_cookie}}"})
                request = {"method": method.upper(), "header": headers, "auth": auth,
                           "url": {"raw": "{{base_url}}" + url_path, "host": ["{{base_url}}"],
                                   "path": [s for s in url_path.split("/") if s], "query": query},
                           "description": operation.get("description", operation.get("summary", "")) +
                               "\n\nExamples are fixtures. Mutating requests create real records if sent to a live server."}
                if value is not None:
                    headers.append({"key": "Content-Type", "value": media_type})
                    request["body"] = {"mode": "raw", "raw": json.dumps(value, indent=2), "options": {"raw": {"language": "json"}}}
                    all_examples.append({"path": path, "method": method, "name": variant, "media_type": media_type, "value": value})
                name = method.upper() + " " + path + (" — " + variant if len(variants) > 1 else "")
                groups.setdefault(operation.get("tags", ["Other"])[0], []).append({"name": name, "request": request, "response": []})
    collection = {
        "info": {"name": "RAN Online GS Payments & Recharge API — " + CONTRACT_DATE,
                 "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
                 "description": "Source-derived examples. Fill session_cookie (value of the player session cookie after login) and admin_jwt (from POST /admin/v1/auth/authenticate). No credentials included. Webhook requests are unsigned in the current implementation and would change real transactions."},
        "variable": [{"key": key, "value": value, "type": "string"} for key, value in variables.items()],
        "item": [{"name": name, "item": items} for name, items in groups.items()],
    }
    write_json(ROOT / "postman.collection.json", collection)
    write_json(ROOT / "request-examples.json", all_examples)
    write_json(ROOT / "postman.environment.json", {
        "name": "RAN Online GS Payments — local values only",
        "values": [{"key": key, "value": value, "enabled": True, "type": "secret" if key in ("session_cookie", "admin_jwt") else "default"}
                   for key, value in variables.items()],
        "_postman_variable_scope": "environment",
    })
    return all_examples


def implemented_routes():
    routes = set()
    for relative, prefix in ROUTE_SOURCES.items():
        source = (REPO / relative).read_text(encoding="utf8").split("#[cfg(test)]")[0]
        for method, path in re.findall(r'^\s*#\[(get|post|put|patch|delete)\("([^"]*)"\)\]', source, re.M):
            routes.add((prefix + path, method))
    return routes


def source_map():
    files = {}
    for relative in SOURCE_FILES:
        path = REPO / relative
        files[relative] = hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
    write_json(ROOT / "source-map.json", {"contract_date": CONTRACT_DATE, "sha256": files})


def check(spec, request_examples):
    from openapi_spec_validator import validate
    from jsonschema import Draft202012Validator
    from referencing import Registry, Resource
    from referencing.jsonschema import DRAFT202012
    validate(spec)
    base = "https://contracts.ranonlinegs.invalid/openapi.json"
    registry = Registry().with_resource(base, Resource.from_contents(spec, default_specification=DRAFT202012))

    def errors_for(schema, value):
        target = {"$ref": base + schema["$ref"]} if "$ref" in schema else {"$id": base, **schema, "components": spec["components"]}
        return [e.message for e in Draft202012Validator(target, registry=registry).iter_errors(value)]

    schema_example_count = 0
    for name, schema in spec["components"]["schemas"].items():
        Draft202012Validator.check_schema(schema)
        for index, example in enumerate(schema.get("examples", [])):
            errors = errors_for({"$ref": "#/components/schemas/" + name}, example)
            assert not errors, name + " example " + str(index) + ": " + "; ".join(errors[:3])
            schema_example_count += 1
    for row in request_examples:
        operation = spec["paths"][row["path"]][row["method"]]
        schema = resolve(spec, operation["requestBody"])["content"][row["media_type"]]["schema"]
        errors = errors_for(schema, row["value"])
        assert not errors, row["method"] + " " + row["path"] + " " + row["name"] + ": " + "; ".join(errors[:3])
    response_example_count = 0
    for path, item in spec["paths"].items():
        for method, operation in item.items():
            if method not in METHODS:
                continue
            for code, response in operation["responses"].items():
                for media_type, media in resolve(spec, response).get("content", {}).items():
                    if media_type != "application/json":
                        continue
                    values = [("example", media["example"])] if "example" in media else []
                    values += [(k, resolve(spec, v)["value"]) for k, v in media.get("examples", {}).items()]
                    for label, value in values:
                        errors = errors_for(media["schema"], value)
                        assert not errors, method.upper() + " " + path + " " + code + " " + label + ": " + "; ".join(errors[:3])
                        response_example_count += 1
    ids = [operation["operationId"] for item in spec["paths"].values() for method, operation in item.items() if method in METHODS]
    assert len(ids) == len(set(ids)), "Duplicate operationId"
    coverage = "source_not_available"
    if all((REPO / relative).is_file() for relative in ROUTE_SOURCES):
        expected = implemented_routes()
        actual = {(path, method) for path, item in spec["paths"].items() for method in item if method in METHODS}
        assert expected == actual, "Route coverage differs: " + repr(sorted(expected ^ actual))
        coverage = "all_implemented_routes_matched"
    result = {"openapi_valid": True, "paths": len(spec["paths"]), "operations": len(ids),
              "schemas": len(spec["components"]["schemas"]), "request_examples_validated": len(request_examples),
              "schema_examples_validated": schema_example_count, "response_examples_validated": response_example_count,
              "route_coverage": coverage}
    write_json(ROOT / "contract-checks.json", result)
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    contract = assemble()
    samples = make_postman(contract)
    source_map()
    if args.check:
        check(contract, samples)
