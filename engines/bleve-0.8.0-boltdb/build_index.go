package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/blevesearch/bleve"
	"github.com/blevesearch/bleve/analysis/analyzer/standard"
	_ "github.com/blevesearch/bleve/config"
	"github.com/blevesearch/bleve/index/store/boltdb"
	"github.com/blevesearch/bleve/index/upsidedown"
)

func main() {
	outputDir := os.Args[1]

	textFieldMapping := bleve.NewTextFieldMapping()
	textFieldMapping.Analyzer = standard.Name
	textFieldMapping.Store = false
	textFieldMapping.IncludeInAll = false

	docMapping := bleve.NewDocumentMapping()
	docMapping.AddFieldMappingsAt("text", textFieldMapping)

	indexMapping := bleve.NewIndexMapping()
	indexMapping.DefaultMapping = docMapping
	indexMapping.DefaultField = "text"

	index, err := bleve.NewUsing(outputDir, indexMapping, upsidedown.Name, boltdb.Name, nil)
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
