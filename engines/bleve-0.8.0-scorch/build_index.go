package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/blevesearch/bleve"
	"github.com/blevesearch/bleve/analysis"
	"github.com/blevesearch/bleve/analysis/token/lowercase"
	"github.com/blevesearch/bleve/analysis/tokenizer/unicode"
	_ "github.com/blevesearch/bleve/config"
	"github.com/blevesearch/bleve/index/scorch"
	"github.com/blevesearch/bleve/registry"
)

func main() {
	outputDir := os.Args[1]

	textFieldMapping := bleve.NewTextFieldMapping()
	textFieldMapping.Analyzer = StandardAnalyzerWithStopWords
	textFieldMapping.Store = false
	textFieldMapping.IncludeInAll = false

	docMapping := bleve.NewDocumentMapping()
	docMapping.AddFieldMappingsAt("text", textFieldMapping)

	indexMapping := bleve.NewIndexMapping()
	indexMapping.DefaultMapping = docMapping
	indexMapping.DefaultField = "text"

	index, err := bleve.NewUsing(outputDir, indexMapping, scorch.Name, scorch.Name, nil)
	defer func() {
		err = index.Close()
		if err != nil {
			fmt.Println(err)
			return
		}
	}()
	if err != nil {
		fmt.Println(err)
		return
	}

	batchSize := 500
	batch := index.NewBatch()

	reader := bufio.NewReader(os.Stdin)
	for {
		lineBytes, err := reader.ReadBytes('\n')
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
					fields := map[string]interface{}{}
					for k, v := range data {
						if k != "id" {
							fields[k] = v
						}
					}
					err = batch.Index(id, fields)
					if err != nil {
						fmt.Println(err)
						return
					}
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
			fields := map[string]interface{}{}
			for k, v := range data {
				if k != "id" {
					fields[k] = v
				}
			}
			err = batch.Index(id, fields)
			if err != nil {
				fmt.Println(err)
				return
			}

			if batch.Size() >= batchSize {
				err := index.Batch(batch)
				if err != nil {
					fmt.Println(err)
					return
				}
				batch = index.NewBatch()
			}
		}
	}

	if batch.Size() > 0 {
		err := index.Batch(batch)
		if err != nil {
			fmt.Println(err)
			return
		}
	}

	docCount, err := index.DocCount()
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println(docCount)

	fmt.Println("Does not support index marge")
}

func NewStandardAnalyzerWithStopWords(config map[string]interface{}, cache *registry.Cache) (*analysis.Analyzer, error) {
	tokenizer, err := cache.TokenizerNamed(unicode.Name)
	if err != nil {
		return nil, err
	}
	toLowerFilter, err := cache.TokenFilterNamed(lowercase.Name)
	if err != nil {
		return nil, err
	}
	rv := analysis.Analyzer{
		Tokenizer: tokenizer,
		TokenFilters: []analysis.TokenFilter{
			toLowerFilter,
		},
	}
	return &rv, nil
}

const StandardAnalyzerWithStopWords = "standard-with-stopwords"

func init() {
	registry.RegisterAnalyzer(StandardAnalyzerWithStopWords, NewStandardAnalyzerWithStopWords)
}
