// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Print format variants
use arrow::csv::writer::WriterBuilder;
use arrow::json::ArrayWriter;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::pretty;
use datafusion::error::{DataFusionError, Result};
use std::str::FromStr;

/// Allow records to be printed in different formats
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrintFormat {
    Csv,
    Tsv,
    Table,
    Json,
}

impl FromStr for PrintFormat {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "csv" => Ok(Self::Csv),
            "tsv" => Ok(Self::Tsv),
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            _ => Err(()),
        }
    }
}

fn print_batches_to_json(batches: &[RecordBatch]) -> Result<String> {
    let mut bytes = vec![];
    {
        let mut writer = ArrayWriter::new(&mut bytes);
        writer.write_batches(batches)?;
        writer.finish()?;
    }
    let formatted = String::from_utf8(bytes)
        .map_err(|e| DataFusionError::Execution(e.to_string()))?;
    Ok(formatted)
}

fn print_batches_with_sep(batches: &[RecordBatch], delimiter: u8) -> Result<String> {
    let mut bytes = vec![];
    {
        let builder = WriterBuilder::new()
            .has_headers(true)
            .with_delimiter(delimiter);
        let mut writer = builder.build(&mut bytes);
        for batch in batches {
            writer.write(batch)?;
        }
    }
    let formatted = String::from_utf8(bytes)
        .map_err(|e| DataFusionError::Execution(e.to_string()))?;
    Ok(formatted)
}

impl PrintFormat {
    /// print the batches to stdout using the specified format
    pub fn print_batches(&self, batches: &[RecordBatch]) -> Result<()> {
        match self {
            Self::Csv => println!("{}", print_batches_with_sep(batches, b',')?),
            Self::Tsv => println!("{}", print_batches_with_sep(batches, b'\t')?),
            Self::Table => pretty::print_batches(batches)?,
            Self::Json => println!("{}", print_batches_to_json(batches)?),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    #[test]
    fn test_from_str() {
        let format = "csv".parse::<PrintFormat>().unwrap();
        assert_eq!(PrintFormat::Csv, format);

        let format = "tsv".parse::<PrintFormat>().unwrap();
        assert_eq!(PrintFormat::Tsv, format);

        let format = "json".parse::<PrintFormat>().unwrap();
        assert_eq!(PrintFormat::Json, format);

        let format = "table".parse::<PrintFormat>().unwrap();
        assert_eq!(PrintFormat::Table, format);
    }

    #[test]
    fn test_from_str_failure() {
        assert_eq!(true, "pretty".parse::<PrintFormat>().is_err());
    }

    #[test]
    fn test_print_batches_with_sep() {
        let batches = vec![];
        assert_eq!("", print_batches_with_sep(&batches, b',').unwrap());

        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
            Field::new("c", DataType::Int32, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Int32Array::from(vec![4, 5, 6])),
                Arc::new(Int32Array::from(vec![7, 8, 9])),
            ],
        )
        .unwrap();

        let batches = vec![batch];
        let r = print_batches_with_sep(&batches, b',').unwrap();
        assert_eq!("a,b,c\n1,4,7\n2,5,8\n3,6,9\n", r);
    }

    #[test]
    fn test_print_batches_to_json_empty() {
        let batches = vec![];
        let r = print_batches_to_json(&batches).unwrap();
        assert_eq!("", r);

        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
            Field::new("c", DataType::Int32, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Int32Array::from(vec![4, 5, 6])),
                Arc::new(Int32Array::from(vec![7, 8, 9])),
            ],
        )
        .unwrap();

        let batches = vec![batch];
        let r = print_batches_to_json(&batches).unwrap();
        assert_eq!("[{\"a\":1,\"b\":4,\"c\":7},{\"a\":2,\"b\":5,\"c\":8},{\"a\":3,\"b\":6,\"c\":9}]", r);
    }
}
