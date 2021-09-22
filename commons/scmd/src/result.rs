// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use cli_table::format::CellFormat;
use cli_table::{Cell, Row, Table};
use flatten_json::flatten;
use serde_json::{json, Value};
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum OutputFormat {
    JSON,
    TABLE,
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "json" | "JSON" => OutputFormat::JSON,
            "table" | "TABLE" => OutputFormat::TABLE,
            _ => OutputFormat::JSON,
        })
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::JSON
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OutputFormat::TABLE => "TABLE",
            OutputFormat::JSON => "JSON",
        };
        write!(f, "{}", s)
    }
}

pub fn print_action_result(
    format: OutputFormat,
    result: Result<Value>,
    console_mode: bool,
) -> Result<()> {
    match format {
        OutputFormat::JSON => {
            // if in console, and is err, print error directly.
            if console_mode && result.is_err() {
                println!("{}", result.unwrap_err().to_string());
                return Ok(());
            }

            let value = match result {
                Ok(value) => {
                    if value.is_null() {
                        value
                    } else {
                        json!({ "ok": value })
                    }
                }
                Err(err) => json!({"err": err.to_string()}),
            };
            print_json(value)
        }
        OutputFormat::TABLE => {
            match result {
                Ok(value) => print_table(value)?,
                // err may contains help message, so directly print err.
                Err(err) => println!("{}", err.to_string()),
            };
            Ok(())
        }
    }
}

pub fn print_json(value: Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let json = serde_json::to_string_pretty(&value)?;
    println!("{}", json);
    Ok(())
}

fn build_rows(values: &[Value]) -> Result<(Vec<Row>, Box<dyn RowBuilder>)> {
    let bold = CellFormat::builder().bold(true).build();
    let mut rows = vec![];
    let mut field_names = Vec::new();
    let is_simple = |value: &Value| value.is_number() || value.is_boolean() || value.is_string();
    let mut exist_not_simple = false;
    for value in values {
        if is_simple(value) {
            rows.push(Row::new(vec![Cell::new("Result", bold)]));
        } else {
            exist_not_simple = true;
            let mut flat = json!({});
            flatten(value, &mut flat, None, true, None)
                .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
            let obj = flat.as_object().expect("must be a object");
            let mut cells = vec![];
            obj.keys().for_each(|key| {
                cells.push(Cell::new(key, bold));
                if !field_names.contains(key) {
                    field_names.push(key.to_string());
                }
            });
            rows.push(Row::new(cells));
        }
    }
    if exist_not_simple {
        Ok((rows, Box::new(ObjectRowBuilder { field_names })))
    } else {
        Ok((rows, Box::new(SimpleRowBuilder)))
    }
}

pub fn print_table(value: Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    match value {
        Value::Array(values) => print_vec_table(values),
        value => print_value_table(value),
    }
}

fn print_vec_table(values: Vec<Value>) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }
    let first = &values[0];
    let first_value = serde_json::to_value(first)?;
    if first_value.is_null() {
        return Ok(());
    }
    if first_value.is_array() {
        bail!("Not support embed array in Action Result.")
    }
    let mut print_rows = vec![];
    let (rows, row_builder) = build_rows(&values)?;
    for (i, row) in rows.into_iter().enumerate() {
        print_rows.push(row);
        print_rows.push(row_builder.build_row(&values[i])?);
    }
    let table = Table::new(print_rows, Default::default())?;
    table.print_stdout()?;
    Ok(())
}

fn print_value_table(value: Value) -> Result<()> {
    let simple_value =
        value.is_number() || value.is_boolean() || value.is_boolean() || value.is_string();
    if simple_value {
        println!("{}", value_to_string(&value));
    } else {
        // value must be a object at here.
        let bold = CellFormat::builder().bold(true).build();
        let mut flat = json!({});
        flatten(&value, &mut flat, None, true, None)
            .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
        let obj = flat.as_object().expect("must be a object");
        let mut rows = vec![];
        for (k, v) in obj {
            let row = Row::new(vec![
                Cell::new(k, bold),
                Cell::new(value_to_string(v).as_str(), Default::default()),
            ]);
            rows.push(row);
        }
        let table = Table::new(rows, Default::default())?;
        table.print_stdout()?;
    }
    Ok(())
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "".to_string(),
        Value::Number(v) => format!("{}", v),
        Value::Bool(v) => format!("{}", v),
        Value::String(v) => v.to_string(),
        v => v.to_string(),
    }
}

trait RowBuilder {
    fn build_row(&self, value: &Value) -> Result<Row>;
}

struct SimpleRowBuilder;

impl RowBuilder for SimpleRowBuilder {
    fn build_row(&self, value: &Value) -> Result<Row> {
        Ok(Row::new(vec![Cell::new(
            value_to_string(value).as_str(),
            Default::default(),
        )]))
    }
}

struct ObjectRowBuilder {
    field_names: Vec<String>,
}

impl RowBuilder for ObjectRowBuilder {
    fn build_row(&self, value: &Value) -> Result<Row> {
        let mut flat = json!({});
        flatten(value, &mut flat, None, true, None)
            .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
        let obj = flat.as_object().expect("must be a object");
        let mut cells = vec![];
        for field in &self.field_names {
            if let Some(v) = obj.get(field) {
                cells.push(Cell::new(value_to_string(v).as_str(), Default::default()));
            }
        }
        Ok(Row::new(cells))
    }
}
