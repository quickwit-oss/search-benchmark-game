package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/blevesearch/bleve"
	"github.com/blevesearch/bleve/analysis"
	"github.com/blevesearch/bleve/analysis/token/lowercase"
	"github.com/blevesearch/bleve/analysis/tokenizer/unicode"
	_ "github.com/blevesearch/bleve/config"
	"github.com/blevesearch/bleve/registry"
)

func main() {
	indexDir := os.Args[1]

	index, err := bleve.Open(indexDir)
	defer func() {
		err = index.Close()
		if err != nil {
			fmt.Sprintf("ERROR: %v", err)
			return
		}
	}()
	if err != nil {
		fmt.Sprintf("ERROR: %v", err)
		return
	}

	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := scanner.Text()
		fields := strings.SplitN(line, "\t", 2)
		command := fields[0]
		query := bleve.NewQueryStringQuery(fields[1])
		searchRequest := bleve.NewSearchRequest(query)

		count := uint64(0)
		switch command {
		case "COUNT":
			searchResult, err := index.Search(searchRequest)
			if err != nil {
				fmt.Sprintf("ERROR: %v", err)
				continue
			}
			count = searchResult.Total
		case "TOP_10":
			searchRequest.Size = 10
			_, err := index.Search(searchRequest)
			if err != nil {
				fmt.Sprintf("ERROR: %v", err)
				continue
			}
			count = 1
		case "TOP_10_COUNT":
			searchRequest.Size = 10
			searchResult, err := index.Search(searchRequest)
			if err != nil {
				fmt.Sprintf("ERROR: %v", err)
				continue
			}
			count = searchResult.Total
		default:
			fmt.Println("UNSUPPORTED")
			continue
		}
		fmt.Println(count)
	}
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
