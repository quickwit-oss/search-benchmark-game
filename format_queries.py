import fileinput
import json
import re
import json
import random

LETTERS_ONLY = re.compile("^[a-z ]+$")
PTN = re.compile("\s+")

def generate_queries(words):
    tags = ["num_token_%i" % len(words)]
    if len(words) > 1:
        intersection = " ".join( ("+" + word) for word in words)
        yield {
            "query": intersection,
            "tags": tags + ["intersection", "global"]
        }
        phrase_query = "\"%s\"" % " ".join(words)
        if random.random() < 0.1: # only 10% of queries
            yield {
                "query": phrase_query,
                "tags": tags + ["phrase"]
            }
        else:
            yield {
                "query": phrase_query,
                "tags": tags
            }
        union_query = " ".join(words)
        yield {
            "query": union_query,
            "tags": tags + ["union", "global"]
        }

for line in fileinput.input():
    (count, query) = PTN.split(line.strip(), 1)
    count = int(count)
    if not LETTERS_ONLY.match(query):
        continue
    words = PTN.split(query)
    for q in generate_queries(words):
        try:
            qdoc = json.dumps(q)
            print(qdoc)
        except:
            pass
