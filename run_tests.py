import json
import subprocess

cts_path = "jsonpath-compliance-test-suite/cts.json"
simdjson_path = "/opt/homebrew/Cellar/simdjson/3.10.1/"

with open(cts_path) as f:
    cts = json.load(f)

tests = [test for test in cts["tests"]
         if "invalid_selector" not in test 
         and "functions" not in test["name"]
         and "filter" not in test["name"]
         and "operators" not in test["name"]]
tests_count = len(tests)
passed_tests = 0

for i, test in enumerate(tests):
    print(f"[{i+1}/{tests_count}] Running test '{test["name"]}'   ", end="")
    failed = False
    with open("prog.cpp", "w") as prog_file:
        res = subprocess.run(["cargo", "r", "--quiet"], input=test["selector"].encode(), stdout=prog_file)
        if res.returncode != 0:
            failed = True
    if not failed:
        res = subprocess.run(
            ["c++", "prog.cpp", "helpers.cpp", "-std=c++11", f"-I{simdjson_path}include", f"-L{simdjson_path}lib", "-lsimdjson", "-Wno-unused-result", "-o", "prog"])
        if res.returncode != 0:
            failed = True
    if not failed:
        input = json.dumps(test["document"], ensure_ascii=False).encode()
        res = subprocess.run("./prog", input=input, capture_output=True)
        if res.returncode != 0:
            failed = True
    if not failed:
        query_result = json.dumps(json.loads(res.stdout.decode()))
        if "result" in test:
            expected_results = [json.dumps(test["result"])]
        else:
            expected_results = [json.dumps(result) for result in test["results"]]
        if query_result not in expected_results:
            failed = True
    if failed:
        print("FAILED")
    else:
        print("OK")
        passed_tests += 1


print(f"\nTotal tests: {tests_count} Passed: {passed_tests}")