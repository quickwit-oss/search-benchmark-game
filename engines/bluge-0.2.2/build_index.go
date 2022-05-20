package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"github.com/blugelabs/bluge"
	"github.com/blugelabs/bluge/analysis"
	"github.com/blugelabs/bluge/analysis/token"
	"github.com/blugelabs/bluge/analysis/tokenizer"
	"github.com/blugelabs/bluge/index"
	"github.com/blugelabs/bluge/index/mergeplan"
	"io"
	"os"
)

var unicodeAnalyzer = &analysis.Analyzer{
	Tokenizer: tokenizer.NewUnicodeTokenizer(),
	TokenFilters: []analysis.TokenFilter{
		token.NewLowerCaseFilter(),
	},
}

func main() {
	outputDir := os.Args[1]

	mergeplan.DefaultMergePlanOptions.MaxSegmentsPerTier = 1
	mergeplan.DefaultMergePlanOptions.SegmentsPerMergeTask = 2

	config := bluge.DefaultConfig(outputDir)
	config.DefaultSearchAnalyzer = unicodeAnalyzer
	writer, err := bluge.OpenWriter(config)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer func() {
		err = writer.Close()
		if err != nil {
			fmt.Println(err)
			return
		}
	}()

	curBatchSize := 0
	maxBatchSize := 20000
	batch := index.NewBatch()

	bufReader := bufio.NewReader(os.Stdin)
	for {
		lineBytes, err := bufReader.ReadBytes('\n')
		if err != nil {
			if err == io.EOF || err == io.ErrClosedPipe {
				if len(lineBytes) > 0 {
					var data map[string]interface{}
					err = json.Unmarshal(lineBytes, &data)
					if err != nil {
						fmt.Println(err)
						return
					}
					id := data["id"].(string)
					doc := bluge.NewDocument(id)
					for k, v := range data {
						if k != "id" {
							doc.AddField(bluge.NewTextField(k, v.(string)).WithAnalyzer(unicodeAnalyzer))
						}
					}
					batch.Insert(doc)
				}
				break
			}
		}

		if len(lineBytes) > 0 {
			var data map[string]interface{}
			err = json.Unmarshal(lineBytes, &data)
			if err != nil {
				fmt.Println(err)
				return
			}
			id := data["id"].(string)
			doc := bluge.NewDocument(id)
			for k, v := range data {
				if k != "id" {
					doc.AddField(bluge.NewTextField(k, v.(string)).WithAnalyzer(unicodeAnalyzer))
				}
			}
			batch.Insert(doc)
			curBatchSize++

			if curBatchSize >= maxBatchSize {
				err := writer.Batch(batch)
				if err != nil {
					fmt.Println(err)
					return
				}
				batch = index.NewBatch()
				curBatchSize = 0
			}
		}
	}

	if curBatchSize > 0 {
		err := writer.Batch(batch)
		if err != nil {
			fmt.Println(err)
			return
		}
	}

	reader, err := writer.Reader()
	if err != nil {
		fmt.Println(err)
		return
	}
	count, err := reader.Count()
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println(count)

	// https://github.com/blugelabs/bluge/issues/15
	fmt.Println("Does not support index marge")
}
