import subprocess
import os
from os import path
import time
import json
import random
import argparse
from collections import defaultdict
import glob
import sys

COMMANDS = os.environ['COMMANDS'].split(' ')
SERVE_TYPE = os.environ.get('SERVE_TYPE', 'serve')

class SearchClient:

    def __init__(self, engine):
        self.engine = engine
        dirname = os.path.split(os.path.abspath(__file__))[0]
        dirname = path.dirname(dirname)
        dirname = path.join(dirname, "engines")
        cwd = path.join(dirname, engine)
        print(cwd)
        self.process = subprocess.Popen(["make", "--no-print-directory", SERVE_TYPE],
            cwd=cwd,
            stdout=subprocess.PIPE,
            stdin=subprocess.PIPE)

    def query(self, query, command):
        query_line = "%s\t%s\n" % (command, query)
        self.process.stdin.write(query_line.encode("utf-8"))
        self.process.stdin.flush()
        recv = self.process.stdout.readline().strip()
        if recv == b"UNSUPPORTED":
            print("Warning: command %s query %s is unsupported by engine %s" % (command, query, self.engine))
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

def save_engine_results(engine, command, engine_results):
    """Save engine_results directly to engine's results directory"""
    dirname = os.path.split(os.path.abspath(__file__))[0]
    dirname = path.dirname(dirname)
    engines_dir = path.join(dirname, "engines")
    engine_dir = path.join(engines_dir, engine)
    
    # Create results directory if it doesn't exist
    results_dir = path.join(engine_dir, "results")
    os.makedirs(results_dir, exist_ok=True)
    
    # Save individual command results
    filename = f"{command}.json"
    filepath = path.join(results_dir, filename)
    
    with open(filepath, "w") as f:
        json.dump(engine_results, f, default=lambda obj: obj.__dict__, sort_keys=True, indent=2)
    
    print(f"Saved {engine}/{command} results to {filepath}")
    return filepath

def merge_results_files(merge_dirs=None):
    """Merge results files from engine directories"""
    all_results = {}
    details = {}
    
    if merge_dirs is None:
        # Default: look in engines directory
        base_dir = "engines"
        if not path.exists(base_dir):
            print(f"Error: Directory {base_dir} does not exist")
            return None
        
        # Find all engine directories in engines/
        engine_dirs = []
        for item in os.listdir(base_dir):
            engine_path = path.join(base_dir, item)
            if path.isdir(engine_path) and path.exists(path.join(engine_path, "results")):
                engine_dirs.append(engine_path)
        
        if not engine_dirs:
            print(f"No engine directories with results found in {base_dir}")
            return None
            
        print(f"Found {len(engine_dirs)} engines in {base_dir}: {[path.basename(d) for d in engine_dirs]}")
    
    else:
        # Use specified directories as engine directories
        engine_dirs = []
        for dir_path in merge_dirs:
            dir_path = os.path.normpath(dir_path)
            if not path.exists(dir_path):
                print(f"Warning: Directory {dir_path} does not exist, skipping")
                continue
            
            if not path.exists(path.join(dir_path, "results")):
                print(f"Warning: {dir_path} does not contain results/ directory, skipping")
                continue
                
            engine_dirs.append(dir_path)
        
        if not engine_dirs:
            print("No valid engine directories specified")
            return None
        
        print(f"Using specified engine directories: {[path.basename(d) for d in engine_dirs]}")
    
    # Process each engine directory
    for engine_dir in engine_dirs:
        engine = path.basename(engine_dir)
        print(f"\nProcessing engine: {engine}")
        
        # Load details
        details_file = path.join(engine_dir, "details.json")
        if path.exists(details_file):
            try:
                with open(details_file, "r") as f:
                    details[engine] = json.load(f)
                print(f"  Loaded details.json")
            except Exception as e:
                print(f"  Error loading details.json: {e}")
                details[engine] = []
        else:
            details[engine] = []
            print(f"  No details.json found")
        
        # Load results
        results_dir = path.join(engine_dir, "results")
        result_files = glob.glob(path.join(results_dir, "*.json"))
        
        if not result_files:
            print(f"  No result files found in {results_dir}")
            continue
        
        for result_file in result_files:
            command = path.basename(result_file)[:-5]  # Remove .json
            try:
                with open(result_file, "r") as f:
                    engine_results = json.load(f)
                
                if command not in all_results:
                    all_results[command] = {}
                
                all_results[command][engine] = engine_results
                print(f"  Loaded {command}.json")
                
            except Exception as e:
                print(f"  Error loading {result_file}: {e}")
    
    if not all_results:
        print("\nNo results loaded")
        return None
    
    # Save merged results
    merged_data = {
        "details": details,
        "results": all_results
    }
    
    with open("results.json", "w") as f:
        json.dump(merged_data, f, default=lambda obj: obj.__dict__, sort_keys=True)
    
    print(f"\nSuccessfully merged {len(details)} engines with {sum(len(v) for v in all_results.values())} command results")
    print("Saved to results.json")
    return "results.json"

WARMUP_TIME = int(os.environ.get('WARMUP_TIME', '60'))
NUM_ITER = int(os.environ.get('NUM_ITER', '10'))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Benchmark search engines')
    parser.add_argument('query_path', nargs='?', help='Path to queries JSON file')
    parser.add_argument('engines', nargs='*', help='Engine names to benchmark')
    parser.add_argument('--merge', action='store_true', help='Merge existing results files instead of running benchmarks')
    parser.add_argument('--merge-dir', action='append', help='Directory to look for engine results (can be used multiple times)')
    
    args = parser.parse_args()
    
    if args.merge:
        # Merge mode: combine existing results files
        if args.merge_dir:
            merge_results_files(args.merge_dir)
        else:
            merge_results_files()
        sys.exit(0)
    
    # Original benchmark mode
    if not args.query_path:
        print("Error: query_path is required for benchmarking mode")
        sys.exit(1)
    
    random.seed(2)
    query_path = args.query_path
    engines = args.engines
    
    if not engines:
        print("Error: No engines specified for benchmarking")
        sys.exit(1)
    
    queries = list(read_queries(query_path))

    details = {}
    for engine in engines:
      dirname = os.path.split(os.path.abspath(__file__))[0]
      dirname = path.dirname(dirname)
      dirname = path.join(dirname, "engines")
      details_file = path.join(dirname, engine, "details.json")
      if os.path.exists(details_file):
        with open(details_file, "r") as f:
          details[engine] = json.loads(f.read())
      else:
        details[engine] = []

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
                        query_idx[query.query] = {"count": -1, "duration": []}
                    elif query_idx[query.query]["count"] != -1:
                        query_idx[query.query]["count"] = count
                        query_idx[query.query]["duration"].append(duration)
                printProgressBar(float(i + 1) / NUM_ITER, prefix = 'Run:   ', suffix = 'Complete', length = 50)
            for query in engine_results:
                query["duration"].sort()
            results_commands[engine] = engine_results
            
            # Save individual engine-command results (engine_results as-is)
            save_engine_results(engine, command, engine_results)
            
            search_client.close()
        print(results_commands.keys())
        results[command] = results_commands
    
    # Save the complete results file (original behavior)
    with open("results.json" , "w") as f:
        json.dump({ "details": details, "results": results }, f, default=lambda obj: obj.__dict__, sort_keys=True)
