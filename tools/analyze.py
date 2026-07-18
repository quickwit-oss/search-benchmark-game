#!/usr/bin/env python3
"""
Analyze results.json from the search benchmark game.

Usage:
    python tools/analyze.py <engine> <command> [--filter <tag>]

Arguments:
    engine    Engine name (e.g., tantivy-main, tantivy-0.26, iresearch-26.03.1)
    command   Benchmark command (e.g., TOP_100, TOP_100_COUNT, COUNT)
    --filter  Optional tag filter: 'union', 'intersection', or 'intersection_union'
              Filters entries whose tags contain the given string.

Examples:
    python tools/analyze.py tantivy-main TOP_100_COUNT
    python tools/analyze.py tantivy-main TOP_100 --filter intersection
    python tools/analyze.py tantivy-0.26 TOP_100_COUNT --filter union
"""

import argparse
import json
import statistics
import sys
from pathlib import Path


def load_results(path: Path) -> dict:
    with open(path) as f:
        return json.load(f)


def summarise(entries: list[dict]) -> dict:
    """Compute summary statistics over all duration values across entries."""
    all_durations = []
    for entry in entries:
        all_durations.extend(entry["duration"])

    if not all_durations:
        return {"count": 0, "query_count": 0}

    sorted_d = sorted(all_durations)
    n = len(sorted_d)
    return {
        "query_count": len(entries),
        "sample_count": n,
        "avg": statistics.mean(sorted_d),
        "min": sorted_d[0],
        "max": sorted_d[-1],
        "p50": sorted_d[n // 2],
        "p95": sorted_d[int(n * 0.95)],
        "p99": sorted_d[int(n * 0.99)],
    }


def main():
    parser = argparse.ArgumentParser(
        description="Analyze search benchmark results.json"
    )
    parser.add_argument("engine", help="Engine name (e.g., tantivy-main)")
    parser.add_argument("command", help="Benchmark command (e.g., TOP_100_COUNT)")
    parser.add_argument(
        "--filter", "-f",
        dest="tag_filter",
        help="Filter entries by tag (e.g., union, intersection, intersection_union)",
    )
    args = parser.parse_args()

    results_path = Path("results.json")
    if not results_path.exists():
        print("Error: results.json not found. Run from the project root.", file=sys.stderr)
        sys.exit(1)

    data = load_results(results_path)

    # Validate command
    if args.command not in data["results"]:
        print(f"Error: unknown command '{args.command}'.", file=sys.stderr)
        print(f"  Available: {list(data['results'].keys())}", file=sys.stderr)
        sys.exit(1)

    results = data["results"][args.command]

    # Validate engine
    if args.engine not in results:
        print(f"Error: unknown engine '{args.engine}'.", file=sys.stderr)
        print(f"  Available: {list(results.keys())}", file=sys.stderr)
        sys.exit(1)

    entries = results[args.engine]

    # Optional tag filter
    if args.tag_filter:
        tag = args.tag_filter
        entries = [e for e in entries if any(tag in t for t in e["tags"])]
        if not entries:
            print(f"No entries matched filter '{tag}'.", file=sys.stderr)
            sys.exit(1)

    summary = summarise(entries)

    # Display
    filter_info = f" (filter: {args.tag_filter})" if args.tag_filter else ""
    print(f"Engine:  {args.engine}")
    print(f"Command: {args.command}{filter_info}")
    print(f"Queries: {summary['query_count']}")
    print(f"Samples: {summary['sample_count']} ({summary['sample_count'] // summary['query_count']} per query)")
    print(f"---")
    print(f"Avg:  {summary['avg']:>10.1f} µs")
    print(f"Min:  {summary['min']:>10.1f} µs")
    print(f"Max:  {summary['max']:>10.1f} µs")
    print(f"P50:  {summary['p50']:>10.1f} µs")
    print(f"P95:  {summary['p95']:>10.1f} µs")
    print(f"P99:  {summary['p99']:>10.1f} µs")


if __name__ == "__main__":
    main()
