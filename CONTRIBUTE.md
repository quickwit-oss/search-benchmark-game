# Adding another engine

Currently only tantivy and lucene are supported, but you can add another search
engine by creating a directory in the engines directory and add a `Makefile`
implementing the following commands :

## clean

Removes all files, including the built index, and your compiled bench program.

## index

Starts a program that will receive documents from stdin and build a search
index. Check out the lucene implementation for reference.

Stemming should be disabled. Tokenization should be something reasonably close to Lucene's
[StandardTokenizer](https://lucene.apache.org/core/7_3_1/core/org/apache/lucene/analysis/standard/StandardTokenizer.html). Discrepancies should be documented in `README.md`.

## serve

Starts a program that will get `tests` from stdin, and output
a result hit count as fast as possible. *If this is not your language's default,
be sure to flush stdout after writing your answer".

The tests consist in a command followed by a query.

The command describes the type of operation that should
be performed. Right now there are three commands

- `COUNT` Outputs the document count.
- `TOP_10`, `TOP_100` and `TOP_1000` compute the top-K elements and output "1"
- `TOP_10_COUNT`, `TOP_100_COUNT` and `TOP_1000_COUNT` compute the top-K documents and the overall count of matching documents, and output the document count.

Scores for these commands should be as close as possible to lucene's BM25.
If BM25  is not available, fall back to TfIdf. If TfIdf is not available,
just implement whatever is available to you. Make sure to document any difference in the `README.md` file.

Queries are expressed in the Lucene query language.

If a command is not supported, just print to stdout "UNSUPPORTED".


# Adding tests

If you would like a command to be added please open an issue.
Wanting to show a specific case where your engine shines is a perfectly valid motivation.

`TOP10` should give some advantage to engines implementing variations of the `WAND` algorithm.
