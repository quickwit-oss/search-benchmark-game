#!/usr/bin/env python3
"""Extract queries from queries.txt into COMMAND\tquery format for do_query.

Usage:
    python3 extract_queries.py [--tag SUBSTRING] [--command CMD] [-o OUTPUT] [queries.txt]

Examples:
    python3 extract_queries.py --tag union --command COUNT -o union_queries.tsv
    python3 extract_queries.py --tag phrase --command TOP_100
"""

import argparse
import json
import sys


def main():
    parser = argparse.ArgumentParser(description="Extract queries in tab-separated format for do_query")
    parser.add_argument("input", nargs="?", default="queries.txt", help="Input queries file (default: queries.txt)")
    parser.add_argument("--tag", default=None, help="Filter queries whose tags contain this substring")
    parser.add_argument("--command", default="COUNT", help="Command to prepend (default: COUNT)")
    parser.add_argument("-o", "--output", default=None, help="Output file (default: stdout)")
    args = parser.parse_args()

    out = open(args.output, "w") if args.output else sys.stdout

    with open(args.input) as f:
        for line in f:
            entry = json.loads(line)
            if args.tag is not None:
                if not any(args.tag in t for t in entry["tags"]):
                    continue
            out.write(f"{args.command}\t{entry['query']}\n")

    if args.output:
        out.close()


if __name__ == "__main__":
    main()
