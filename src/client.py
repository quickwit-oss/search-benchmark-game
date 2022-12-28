import subprocess
import os
from os import path
import time
import json
import random
from collections import defaultdict

COMMANDS = os.environ['COMMANDS'].split(' ')

class SearchClient:

    def __init__(self, engine, unsupported_queries):
        self.engine = engine
        self.unsupported_queries = unsupported_queries
        dirname = os.path.split(os.path.abspath(__file__))[0]
        dirname = path.dirname(dirname)
        dirname = path.join(dirname, "engines")
        cwd = path.join(dirname, engine)
        print(cwd)
        self.process = subprocess.Popen(["make", "--no-print-directory", "serve"],
            cwd=cwd,
            stdout=subprocess.PIPE,
            stdin=subprocess.PIPE)

    def query(self, query, command):
        if query in unsupported_queries:
            return None
        query_line = "%s\t%s\n" % (command, query)
        self.process.stdin.write(query_line.encode("utf-8"))
        self.process.stdin.flush()
        recv = self.process.stdout.readline().strip()
        if recv == b"UNSUPPORTED":
            return None
        cnt = int(recv)
        return cnt

    def close(self):
        self.process.stdin.close()
        self.process.stdout.close()

def drive(queries, client, command):
    for query in queries:
        start = time.monotonic()
        count = client.query(query.query, command)
        stop = time.monotonic()
        duration = int((stop - start) * 1e6)
        yield (query, count, duration)

class Query(object):
    def __init__(self, query, tags):
        self.query = query
        self.tags = tags

def read_queries(query_path):
    for q in open(query_path):
        c = json.loads(q)
        yield Query(c["query"], c["tags"])

WARMUP_ITER = 1
NUM_ITER = 3

def filter_non_range_queries(queries):
  return [query for query in queries if 'range' not in query.tags]

def get_range_queries(queries):
    range_queries = set()
    for query in queries:
        if 'range' in query.tags:
            range_queries.add(query.query)
    return range_queries

if __name__ == "__main__":
    import sys
    random.seed(2)
    query_path = sys.argv[1]
    engines = sys.argv[2:]
    range_query_enabled_engines = os.environ['RANGE_QUERY_ENABLED_ENGINES'].split(" ")
    range_query_enabled_engines  = [engine.strip() for engine in range_query_enabled_engines]
    queries = list(read_queries(query_path))
    # non_range_queries = filter_non_range_queries(queries)
    range_queries = get_range_queries(queries)
    results = {}
    for command in COMMANDS:
        results_commands = {}
        for engine in engines:
            engine_results = []
            query_idx = {}
            if engine in range_query_enabled_engines:
                unsupported_queries = set()
            else:
                unsupported_queries = range_queries

            for query in queries:
                query_result = {
                    "query": query.query,
                    "tags": query.tags,
                    "count": 0,
                    "duration": []
                }
                query_idx[query.query] = query_result
                engine_results.append(query_result)
            print("======================")
            print("BENCHMARKING %s %s" % (engine, command))
            search_client = SearchClient(engine, unsupported_queries)
            print("--- Warming up ...")
            queries_shuffled = list(queries[:])
            random.seed(2)
            random.shuffle(queries_shuffled)
            for i in range(WARMUP_ITER):
                for _ in drive(queries_shuffled, search_client, command):
                    pass
            for i in range(NUM_ITER):
                print("- Run #%s of %s" % (i + 1, NUM_ITER))
                for (query, count, duration) in drive(queries_shuffled, search_client, command):
                    if count is None:
                        query_idx[query.query] = {count: -1, duration: []}
                    else:
                        query_idx[query.query]["count"] = count
                        query_idx[query.query]["duration"].append(duration)
            for query in engine_results:
                query["duration"].sort()
            results_commands[engine] = engine_results
            search_client.close()
        print(results_commands.keys())
        results[command] = results_commands
    with open("results.json" , "w") as f:
        json.dump(results, f, default=lambda obj: obj.__dict__)
