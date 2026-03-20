### About indexing speed

Although iresearch supports parallel indexing (as do Lucene and Tantivy), we intentionally use single-threaded indexing for benchmarks. This makes indexing slower, but keeps results more consistent and comparable across engines.

### How to collect pgo?

It's mostly useful for not manually optimized code paths.
It was with intersection_union which was improved by 15% before we optimized it manually.
Fot other queries it gives around 1-2%, so mostly don't cost it.

```bash
cd ./engines/iresearch
rm -rf ./pgo ./cs_pgo
make clean
make compile_gen_pgo
cd ./../..
SERVE_TYPE=serve_gen_pgo ENGINES=iresearch make bench
cd ./engines/iresearch/pgo
llvm-profdata-21 merge -output=./code.profdata ./code-*.profraw
cd ./..
# make clean
# make compile_gen_cs_pgo
# cd ./../..
# SERVE_TYPE=serve_gen_cs_pgo ENGINES=iresearch make bench
# cd ./engines/iresearch/cs_pgo
# llvm-profdata-21 merge -output=./code.profdata ./../pgo/code.profdata ./code-*.profraw
# mv ./code.profdata ./../pgo/code.profdata
# cd ./..
make clean
make compile_use_pgo
cd ./../..
ENGINES=iresearch make bench
```
