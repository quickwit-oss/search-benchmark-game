import React from 'react';
import $ from 'jquery';
import ReactDOM from 'react-dom';
import './style.scss'
import * as serviceWorker from './serviceWorker';

function formatPercentVariation(p) {
  if (p !== undefined) {
    return "+" + (p * 100).toFixed(1) + " %";
  } else {
    return "";
  }
}

function numberWithCommas(x) {
  x = x.toString();
  let pattern = /(-?\d+)(\d{3})/
  while (pattern.test(x)) {
    x = x.replace(pattern, "$1,$2");
  }
  return x;
}

function stats(timings) {
  let median = timings[(timings.length / 2) | 0];
  let mean = timings.reduce(((pv, cv) => pv + cv), 0) / timings.length;
  return {
    "median": median,
    "mean": mean,
    "min": timings[0],
    "max": timings[timings.length - 1]
  };
}

function aggregate(query) {
  if (query.duration.length === 0) {
    return { query: query.query, className: "unsupported", unsupported: true }
  }
  var res = stats(query.duration);
  res.count = query.count;
  res.query = query.query;
  return res;
}


class Benchmark extends React.Component {

  constructor(props) {
    super(props);
    this.state = {
      mode: "TOP_10",
      tag: null
    };
  }

  handleChangeMode(evt) {
    this.setState({ mode: evt.target.value });
  }

  handleChangeTag(evt) {
    var tag = evt.target.value;
    if (tag === "ALL") {
      this.setState({ "tag": null });
    } else {
      this.setState({ "tag": tag });
    }
  }

  filterQueries(queries) {
    let tag = this.state.tag;
    if (tag !== undefined) {
      return queries.filter(query => query.tags.indexOf(tag) >= 0);
    } else {
      return queries;
    }
  }

  generateDataView() {
    var engines = {}
    var queries = {}
    var mode_data = this.props.data[this.state.mode];
    for (var engine in mode_data) {
      var engine_queries = mode_data[engine];
      engine_queries = Array.from(this.filterQueries(engine_queries));
      engine_queries = engine_queries.map(aggregate);
      var total = 0
      var unsupported = false
      for (var query of engine_queries) {
        if (query.unsupported) {
          unsupported = true;
        } else {
          total += query.min;
        }
      }
      if (unsupported) {
        total = undefined;
      } else {
        total = (total / engine_queries.length) | 0;
      }
      engines[engine] = total;
      for (let query of engine_queries) {
        var query_data = {};
        if (queries[query.query] !== undefined) {
          query_data = queries[query.query];
        }
        query_data[engine] = query
        queries[query.query] = query_data
      }
    }

    for (let query in queries) {
      let query_data = queries[query];
      var min_engine = null;
      var min_microsecs = 0;
      var max_engine = null;
      var max_microsecs = 0;
      for (let engine in query_data) {
        var engine_data = query_data[engine];
        if (engine_data.unsupported)
          continue;
        if (min_engine == null || engine_data.min < min_microsecs) {
          min_engine = engine;
          min_microsecs = engine_data.min;
        }
        if (max_engine == null || engine_data.min > max_microsecs) {
          max_engine = engine;
          max_microsecs = engine_data.min;
        }
      }
      for (let engine in query_data) {
        let engine_data = query_data[engine];
        if (engine_data.unsupported) continue;
        if (engine !== min_engine) {
          engine_data.variation = (engine_data.min - min_microsecs) / min_microsecs;
        }
      }
      if (min_engine != null) {
        // Only useful if at least one engine supports this query 
        query_data[min_engine].className = "fastest";
        query_data[max_engine].className = "slowest";
      }
    }
    return { engines, queries };
  }

  render() {
    var data_view = this.generateDataView();
    return <div>
      <form>
        <fieldset>
          <label htmlFor="collectionField">Collection type</label>
          <select id="collectionField" onChange={evt => this.handleChangeMode(evt)}>
            {this.props.modes.map((mode) => <option value={mode} key={mode}>{mode}</option>)}
          </select>
          <label htmlFor="queryTagField">Type of Query</label>
          <select id="queryTagField" onChange={(evt) => this.handleChangeTag(evt)}>
            <option value="ALL" key="all">ALL</option>
            {this.props.tags.map((tag) => <option value={tag} key={tag}>{tag}</option>)}
          </select>
        </fieldset>
      </form>
      <hr />
      <table>
        <thead>
          <tr>
            <th>Query</th>
            {
              Object.keys(data_view.engines).map((engine) => <th key={"col-" + engine}>{engine}</th>)
            }
          </tr>
        </thead>
        <tbody>
          <tr className="average-row">
            <td>AVERAGE</td>
            {
              Object.entries(data_view.engines).map(kv => {
                var engine = kv[0];
                var engine_stats = kv[1];
                if (engine_stats !== undefined) {
                  return <td key={"result-" + engine}>
                    {numberWithCommas(engine_stats)} μs
                </td>;
                } else {
                  return <td key={"result-" + engine}>
                    Some Unsupported Queries
                </td>;
                }
              })
            }
          </tr>
          {
            Object.entries(data_view.queries).map(kv => {
              var query = kv[0];
              var engine_queries = kv[1];
              return <tr>
                <td>{query}</td>
                {
                  Object.keys(data_view.engines).map(engine => {
                    var cell_data = engine_queries[engine];
                    if (cell_data.unsupported) {
                      return <td className={"data " + cell_data.className}></td>;
                    } else {
                      return <td className={"data " + cell_data.className}>
                        <div className="timing">{numberWithCommas(cell_data.min)}  μs</div>
                        <div className="timing-variation">{formatPercentVariation(cell_data.variation)}</div>
                        <div className="count">{numberWithCommas(cell_data.count)} docs</div>
                      </td>;
                    }
                  })
                }
              </tr>
            })
          }
        </tbody>
      </table>
    </div>;
  }

}

$(function () {
  $.getJSON(process.env.PUBLIC_URL + "/results.json", (data) => {
    var modes = [];
    var engines = [];
    var tags_set = new Set();
    for (var mode in data) {
      modes.push(mode);
    }
    for (var engine in data[modes[0]]) {
      engines.push(engine);
    }
    for (var query of data[modes[0]][engines[0]]) {
      for (var tag of query.tags) {
        tags_set.add(tag);
      }
    }
    var tags = Array.from(tags_set);
    tags.sort();
    var el = document.getElementById("app-container");
    ReactDOM.render(<React.StrictMode>
      <Benchmark data={data} tags={tags} modes={modes} engines={engines} />
    </React.StrictMode>, el);
  });
});

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
