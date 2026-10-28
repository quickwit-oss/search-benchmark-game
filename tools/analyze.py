#!/usr/bin/env python3
"""Analyze and compare search-benchmark-game results.

Examples:
    python tools/analyze.py tantivy-main
    python tools/analyze.py tantivy-main --command TOP_100_COUNT
    python tools/analyze.py tantivy-perf --command TOP_100_COUNT --compare tantivy-main
    python tools/analyze.py tantivy-perf --compare tantivy-main --all-filters
    python tools/analyze.py tantivy-perf --compare tantivy-main --format tsv
"""

import argparse
import json
import math
import statistics
import sys
from pathlib import Path


FILTERS = (
    None,
    "union",
    "intersection",
    "intersection_union",
    "pure_union",
    "pure_intersection",
)
PURE_FILTER_TAGS = {
    "pure_union": "union",
    "pure_intersection": "intersection",
}


def fail(message: str) -> None:
    print(f"Error: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_results(path: Path) -> dict:
    try:
        with path.open() as file:
            return json.load(file)
    except FileNotFoundError:
        fail(f"{path} not found")
    except (OSError, json.JSONDecodeError) as error:
        fail(f"could not read {path}: {error}")


def filter_entries(entries: list[dict], tag_filter: str | None) -> list[dict]:
    if tag_filter is None:
        return entries
    if exact_tag := PURE_FILTER_TAGS.get(tag_filter):
        return [entry for entry in entries if exact_tag in entry.get("tags", [])]
    # Deliberately retain the historical substring behavior: the `union` and
    # `intersection` views also contain `intersection_union` queries.
    return [
        entry
        for entry in entries
        if any(tag_filter in tag for tag in entry.get("tags", []))
    ]


def geomean(values: list[float]) -> float:
    if not values or any(value <= 0 for value in values):
        return float("nan")
    return math.exp(sum(math.log(value) for value in values) / len(values))


def query_averages(entries: list[dict]) -> dict[str, float]:
    averages = {}
    for entry in entries:
        query = entry["query"]
        if query in averages:
            fail(f"duplicate query in results: {query!r}")
        durations = entry["duration"]
        if not durations:
            fail(f"query has no duration samples: {query!r}")
        averages[query] = statistics.mean(durations)
    return averages


def percentile(values: list[float], fraction: float) -> float:
    """Match the UI's Math.round((n - 1) * fraction) selection."""
    return sorted(values)[math.floor((len(values) - 1) * fraction + 0.5)]


def summarise(entries: list[dict]) -> dict:
    """Compute the overall average and percentiles across query averages."""
    all_durations = [duration for entry in entries for duration in entry["duration"]]
    if not all_durations:
        fail("no entries matched")

    sorted_durations = sorted(all_durations)
    averages = list(query_averages(entries).values())
    sample_count = len(sorted_durations)
    return {
        "query_count": len(entries),
        "sample_count": sample_count,
        "samples_per_query": sample_count / len(entries),
        "avg": statistics.mean(sorted_durations),
        "min": sorted_durations[0],
        "max": sorted_durations[-1],
        "p50": percentile(averages, 0.50),
        "p90": percentile(averages, 0.90),
        "p95": percentile(averages, 0.95),
        "p99": percentile(averages, 0.99),
        "suite_total": sum(averages),
        "geomean_query": geomean(averages),
    }


def compare(candidate_entries: list[dict], baseline_entries: list[dict]) -> dict:
    candidate_averages = query_averages(candidate_entries)
    baseline_averages = query_averages(baseline_entries)
    if candidate_averages.keys() != baseline_averages.keys():
        missing_candidate = baseline_averages.keys() - candidate_averages.keys()
        missing_baseline = candidate_averages.keys() - baseline_averages.keys()
        fail(
            "candidate and baseline query sets differ "
            f"(missing candidate={len(missing_candidate)}, "
            f"missing baseline={len(missing_baseline)})"
        )

    queries = list(candidate_averages)
    candidate_values = [candidate_averages[query] for query in queries]
    baseline_values = [baseline_averages[query] for query in queries]
    baseline_counts = {entry["query"]: entry["count"] for entry in baseline_entries}
    count_mismatches = sum(
        entry["count"] != baseline_counts[entry["query"]]
        for entry in candidate_entries
    )

    candidate_total = sum(candidate_values)
    baseline_total = sum(baseline_values)
    ratios = [
        candidate / baseline
        for candidate, baseline in zip(candidate_values, baseline_values)
    ]
    return {
        "query_count": len(queries),
        "candidate_p50": percentile(candidate_values, 0.50),
        "baseline_p50": percentile(baseline_values, 0.50),
        "candidate_suite_total": candidate_total,
        "baseline_suite_total": baseline_total,
        "suite_delta_pct": (candidate_total / baseline_total - 1.0) * 100.0,
        "geomean_delta_pct": (geomean(ratios) - 1.0) * 100.0,
        "count_mismatches": count_mismatches,
    }


def validate_engine(command_results: dict, engine: str, command: str) -> None:
    if engine not in command_results:
        fail(
            f"unknown engine {engine!r} for {command}; "
            f"available: {list(command_results)}"
        )


def build_rows(data: dict, args: argparse.Namespace) -> list[dict]:
    available_commands = data.get("results", {})
    commands = list(available_commands) if args.command == "all" else [args.command]
    for command in commands:
        if command not in available_commands:
            fail(f"unknown command {command!r}; available: {list(available_commands)}")

    filters = FILTERS if args.all_filters else (args.tag_filter,)
    rows = []
    for command in commands:
        command_results = available_commands[command]
        validate_engine(command_results, args.engine, command)
        if args.compare:
            validate_engine(command_results, args.compare, command)

        for tag_filter in filters:
            candidate_entries = filter_entries(command_results[args.engine], tag_filter)
            if not candidate_entries:
                fail(f"no entries matched filter {tag_filter!r} for {command}")
            row = {
                "command": command,
                "filter": tag_filter or "all",
                "engine": args.engine,
            }
            if args.compare:
                baseline_entries = filter_entries(command_results[args.compare], tag_filter)
                if not baseline_entries:
                    fail(
                        f"no baseline entries matched filter {tag_filter!r} for {command}"
                    )
                row["baseline"] = args.compare
                row.update(compare(candidate_entries, baseline_entries))
            else:
                row.update(summarise(candidate_entries))
            rows.append(row)
    return rows


def print_single_text(row: dict) -> None:
    filter_info = f" (filter: {row['filter']})" if row["filter"] != "all" else ""
    print(f"Engine:  {row['engine']}")
    print(f"Command: {row['command']}{filter_info}")
    print(f"AVERAGE: {row['avg']:>9.1f} µs")
    print(f"Min:  {row['min']:>12.1f} µs")
    print(f"Max:  {row['max']:>12.1f} µs")
    print(f"P50:  {row['p50']:>12.1f} µs")
    print(f"P90:  {row['p90']:>12.1f} µs")
    print(f"P95:  {row['p95']:>12.1f} µs")
    print(f"P99:  {row['p99']:>12.1f} µs")


def print_comparison_text(rows: list[dict]) -> None:
    print(
        f"{'COMMAND':<15} {'FILTER':<20} {'Q':>4} "
        f"{'CAND P50':>10} {'BASE P50':>10} "
        f"{'CAND TOTAL':>12} {'BASE TOTAL':>12} "
        f"{'SUITE Δ':>9} {'GEO Δ':>9} {'BAD':>4}"
    )
    for row in rows:
        print(
            f"{row['command']:<15} {row['filter']:<20} {row['query_count']:>4} "
            f"{row['candidate_p50']:>10.1f} "
            f"{row['baseline_p50']:>10.1f} "
            f"{row['candidate_suite_total']:>12.1f} "
            f"{row['baseline_suite_total']:>12.1f} "
            f"{row['suite_delta_pct']:>+8.2f}% "
            f"{row['geomean_delta_pct']:>+8.2f}% "
            f"{row['count_mismatches']:>4}"
        )
    print(f"\nCandidate: {rows[0]['engine']}  Baseline: {rows[0]['baseline']}")
    print("Negative deltas mean the candidate is faster.")


def print_tsv(rows: list[dict]) -> None:
    fields = list(rows[0])
    print("\t".join(fields))
    for row in rows:
        print("\t".join(str(row[field]) for field in fields))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("engine", help="candidate engine name")
    parser.add_argument(
        "--command", default="all", help="benchmark command, or 'all' (default: all)"
    )
    parser.add_argument(
        "--compare", "-c", metavar="ENGINE", help="baseline engine to compare against"
    )
    filter_group = parser.add_mutually_exclusive_group()
    filter_group.add_argument(
        "--filter",
        "-f",
        dest="tag_filter",
        help="filter entries by tag substring",
    )
    filter_group.add_argument(
        "--all-filters",
        action="store_true",
        help=(
            "report all, substring union/intersection, intersection_union, "
            "and exact-tag pure_union/pure_intersection views"
        ),
    )
    parser.add_argument(
        "--results",
        type=Path,
        default=Path("results.json"),
        help="results JSON path (default: results.json)",
    )
    parser.add_argument(
        "--format",
        choices=("text", "tsv", "json"),
        default="text",
        help="output format",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    rows = build_rows(load_results(args.results), args)
    if args.format == "json":
        print(json.dumps(rows, indent=2))
    elif args.format == "tsv":
        print_tsv(rows)
    elif args.compare:
        print_comparison_text(rows)
    else:
        for index, row in enumerate(rows):
            if index:
                print()
            print_single_text(row)


if __name__ == "__main__":
    main()
