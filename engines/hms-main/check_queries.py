#!/usr/bin/env python3
# Copyright 2024-2026 WritersLogic Contributors
# SPDX-License-Identifier: Apache-2.0

"""Compare the actual line protocol with an independent token-sequence evaluator."""

import argparse
import json
import os
from pathlib import Path
import random
import re
import subprocess
import tempfile


def matches(text, query):
    words = text.split()
    required, optional, excluded = [], [], []
    for token in re.finditer(r'([+-]?)(?:"([a-z ]+)"|([a-z]+))', query):
        occur, phrase, term = token.groups()
        sequence = (phrase or term).split()
        found = any(words[i : i + len(sequence)] == sequence for i in range(len(words)))
        (required if occur == "+" else excluded if occur == "-" else optional).append(found)
    return not any(excluded) and (all(required) if required else any(optional))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--engine", type=Path, default=Path(__file__).parent)
    parser.add_argument("--queries", type=Path, default=Path(__file__).parents[2] / "queries.txt")
    args = parser.parse_args()
    rng = random.Random(17)
    texts = [" ".join(rng.choices(["the", "who", "uk", "red", "blue", "green", "fox"], k=rng.randrange(1, 40))) for _ in range(129)]
    texts += [" ".join(["filler"] * 4095 + ["the", "who", "uk"]), "the who who uk", "red red red blue"]
    queries = [json.loads(line)["query"] for line in args.queries.read_text().splitlines()]
    queries += ["", "-the", "+red -red", '"red red"', '+"the who" +uk', "+absent red", "+red absent", "red red", "+red +red"]
    engine = args.engine.resolve()
    with tempfile.TemporaryDirectory(prefix="hms-protocol-") as directory:
        corpus = "".join(json.dumps({"id": str(i), "text": text}) + "\n" for i, text in enumerate(texts))
        env = dict(os.environ, HMS_DIFFERENTIAL="1", RAYON_NUM_THREADS="4")
        built = subprocess.run([engine / "target/release/build_index", directory], input=corpus, text=True, capture_output=True, env=env)
        if built.returncode:
            raise RuntimeError(built.stderr)
        commands = ["COUNT", "TOP_10_COUNT", "TOP_100_COUNT", "TOP_1000_COUNT", "TOP_10", "TOP_1000"]
        lines = [(command, query) for command in commands for query in queries]
        expected_answers = [sum(matches(text, query) for text in texts) if command.endswith("COUNT") else 1 for command, query in lines]
        for execution in ["auto", "daat", "wand"]:
            served = subprocess.run([engine / "target/release/do_query", directory], input="".join(f"{command}\t{query}\n" for command, query in lines), text=True, capture_output=True, env=dict(env, HMS_LEXICAL_EXECUTION=execution))
            if served.returncode:
                raise RuntimeError(served.stderr)
            answers = served.stdout.splitlines()
            if len(answers) != len(lines):
                raise AssertionError((len(answers), len(lines)))
            for (command, query), answer, expected in zip(lines, answers, expected_answers):
                if int(answer) != expected:
                    raise AssertionError((execution, command, query, int(answer), expected))
        print(json.dumps({"documents": len(texts), "queries": len(queries), "protocol_answers": len(lines)*3, "executions": ["auto", "daat", "wand"], "oracle": "independent token windows", "optimized_vs_exhaustive": "exact", "status": "passed"}))


if __name__ == "__main__":
    main()
