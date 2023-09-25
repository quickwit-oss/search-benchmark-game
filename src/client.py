import subprocess
import os
from os import path
import time
import json
import random
from collections import defaultdict

COMMANDS = os.environ['COMMANDS'].split(' ')

class SearchClient:

    def __init__(self, engine):
        self.engine = engine
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

# Print progress, borrowed from https://stackoverflow.com/questions/3173320/text-progress-bar-in-terminal-with-block-characters
def printProgressBar (progress, prefix = '', suffix = '', decimals = 1, length = 100, fill = '█', printEnd = "\r"):
    """
    Call in a loop to create terminal progress bar
    @params:
        progress    - Required  : current progress in [0,1] (Float)
        prefix      - Optional  : prefix string (Str)
        suffix      - Optional  : suffix string (Str)
        decimals    - Optional  : positive number of decimals in percent complete (Int)
        length      - Optional  : character length of bar (Int)
        fill        - Optional  : bar fill character (Str)
        printEnd    - Optional  : end character (e.g. "\r", "\r\n") (Str)
    """
    percent = ("{0:." + str(decimals) + "f}").format(100 * progress)
    filledLength = int(length * progress)
    bar = fill * filledLength + '-' * (length - filledLength)
    print(f'\r{prefix} |{bar}| {percent}% {suffix}', end = printEnd)
    # Print New Line on Complete
    if progress >= 1:
        print()

WARMUP_TIME = 10 * 60 # 10 minutes
NUM_ITER = 10

if __name__ == "__main__":
    import sys
    random.seed(2)
    query_path = sys.argv[1]
    engines = sys.argv[2:]
    queries = list(read_queries(query_path))
    results = {}
    for command in COMMANDS:
        results_commands = {}
        for engine in engines:
            engine_results = []
            query_idx = {}
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
            search_client = SearchClient(engine)
            queries_shuffled = list(queries[:])
            random.seed(2)
            random.shuffle(queries_shuffled)
            warmup_start = time.monotonic()
            printProgressBar(0, prefix = 'Warmup:', suffix = 'Complete', length = 50)
            while True:
                for _ in drive(queries_shuffled, search_client, command):
                    pass
                progress = min(1, (time.monotonic() - warmup_start) / WARMUP_TIME)
                printProgressBar(progress, prefix = 'Warmup:', suffix = 'Complete', length = 50)
                if progress == 1:
                    break
            printProgressBar(0, prefix = 'Run:   ', suffix = 'Complete', length = 50)
            for i in range(NUM_ITER):
                for (query, count, duration) in drive(queries_shuffled, search_client, command):
                    if count is None:
                        query_idx[query.query] = {count: -1, duration: []}
                    else:
                        query_idx[query.query]["count"] = count
                        query_idx[query.query]["duration"].append(duration)
                printProgressBar(float(i + 1) / NUM_ITER, prefix = 'Run:   ', suffix = 'Complete', length = 50)
            for query in engine_results:
                query["duration"].sort()
            results_commands[engine] = engine_results
            search_client.close()
        print(results_commands.keys())
        results[command] = results_commands
    with open("results.json" , "w") as f:
        json.dump(results, f, default=lambda obj: obj.__dict__)
