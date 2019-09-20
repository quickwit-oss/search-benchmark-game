package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/blevesearch/bleve"
)

func main() {
	indexDir := os.Args[1]

	index, err := bleve.Open(indexDir)
	defer func() {
		err = index.Close()
		if err != nil {
			fmt.Println("ERROR")
			return
		}
	}()
	if err != nil {
		fmt.Println("ERROR")
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
				fmt.Println("ERROR")
				continue
			}
			count = searchResult.Total
		case "TOP_10":
			searchRequest.Size = 10
			_, err := index.Search(searchRequest)
			if err != nil {
				fmt.Println("ERROR")
				continue
			}
			count = 1
		case "TOP_10_COUNT":
			searchRequest.Size = 10
			searchResult, err := index.Search(searchRequest)
			if err != nil {
				fmt.Println("ERROR")
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
