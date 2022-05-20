package main

import (
	"bufio"
	"context"
	"fmt"
	"github.com/blugelabs/bluge"
	"github.com/blugelabs/bluge/analysis"
	"github.com/blugelabs/bluge/analysis/token"
	"github.com/blugelabs/bluge/analysis/tokenizer"
	"github.com/blugelabs/bluge/search/aggregations"
	querystr "github.com/blugelabs/query_string"
	"os"
	"strings"
)

var unicodeAnalyzer = &analysis.Analyzer{
	Tokenizer: tokenizer.NewUnicodeTokenizer(),
	TokenFilters: []analysis.TokenFilter{
		token.NewLowerCaseFilter(),
	},
}

func main() {
	indexDir := os.Args[1]

	config := bluge.DefaultConfig(indexDir)
	config.DefaultSearchField = "text"
	config.DefaultSearchAnalyzer = unicodeAnalyzer
	writer, err := bluge.OpenWriter(config)
	if err != nil {
		fmt.Printf("ERROR: %v", err)
		return
	}
	defer writer.Close()

	index, err := writer.Reader()
	defer func() {
		err = index.Close()
		if err != nil {
			fmt.Printf("ERROR: %v\n", err)
			return
		}
	}()
	if err != nil {
		fmt.Printf("ERROR: %v\n", err)
		return
	}

	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := scanner.Text()
		fields := strings.SplitN(line, "\t", 2)
		command := fields[0]
		query, err := querystr.ParseQueryString(fields[1], querystr.DefaultOptions().WithDefaultAnalyzer(unicodeAnalyzer))
		if err != nil {
			fmt.Printf("ERROR: %v\n", err)
			continue
		}

		count := uint64(0)
		switch command {
		case "TOP_10":
			_, err := index.Search(context.Background(), bluge.NewTopNSearch(10, query))
			if err != nil {
				fmt.Printf("ERROR: %v", err)
				continue
			}
			count = 1
		case "TOP_10_COUNT":
			req := bluge.NewTopNSearch(10, query)
			req.Aggregations().Add("count", aggregations.CountMatches())
			searchResult, err := index.Search(context.Background(), req)
			if err != nil {
				fmt.Printf("ERROR: %v\n", err)
				continue
			}
			count = searchResult.Aggregations().Count()
		default:
			fmt.Println("UNSUPPORTED")
			continue
		}
		fmt.Println(count)
	}
}
