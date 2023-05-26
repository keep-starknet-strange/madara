from json import load

with open("report/benchmark/metrics.json") as f:
    old_benchmark = load(f)
with open("benchmarking/reports/metrics.json") as f:
    new_benchmark = load(f)
performance_change = (new_benchmark["avgTps"] - old_benchmark["avgTps"]) / old_benchmark["avgTps"] * 100
print("Old benchmark tps: ", old_benchmark["avgTps"])
print("New benchmark tps: ", new_benchmark["avgTps"])
print("Performances changed by: ", performance_change, "%")
assert performance_change >= -10, "Performances degraded by more than 10%"
