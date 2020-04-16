// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use cli_table::format::CellFormat;
use cli_table::{Cell, Row, Table};
use flatten_json::flatten;
use serde_json::{json, Value};
use std::str::FromStr;

pub enum OutputFormat {
    JSON,
    TABLE,
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "json" => OutputFormat::JSON,
            _ => OutputFormat::TABLE,
        })
    }
}

pub fn print_action_result(value: Value, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::JSON => print_json(value),
        OutputFormat::TABLE => print_table(value),
    }
}

pub fn print_json(value: Value) -> Result<()> {
    let result = json!({ "result": value });
    let json = serde_json::to_string_pretty(&result).map_err(|e| Into::<anyhow::Error>::into(e))?;
    println!("{}", json);
    Ok(())
}

fn head_row(first_value: &Value) -> Result<(Row, Box<dyn RowBuilder>)> {
    let bold = CellFormat::builder().bold(true).build();
    let simple_value = first_value.is_number()
        || first_value.is_boolean()
        || first_value.is_boolean()
        || first_value.is_string();
    if simple_value {
        let row = Row::new(vec![Cell::new("Result", bold)]);
        Ok((row, Box::new(SimpleRowBuilder)))
    } else {
        let mut flat = json!({});
        flatten(first_value, &mut flat, None, true)
            .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
        let obj = flat.as_object().expect("must be a object");
        let mut cells = vec![];
        let mut field_names = vec![];
        for (k, _v) in obj {
            field_names.push(k.to_string());
        }
        for field_name in &field_names {
            cells.push(Cell::new(field_name, bold));
        }
        let row = Row::new(cells);
        Ok((row, Box::new(ObjectRowBuilder { field_names })))
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
    let (head_row, row_builder) = head_row(&first_value)?;
    let mut rows = vec![];
    rows.push(head_row);
    rows.push(row_builder.build_row(&first_value)?);
    for value in values[1..].iter() {
        rows.push(row_builder.build_row(&value)?);
    }
    let table = Table::new(rows, Default::default())?;
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
        flatten(&value, &mut flat, None, true)
            .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
        let obj = flat.as_object().expect("must be a object");
        let mut rows = vec![];
        for (k, v) in obj {
            let row = Row::new(vec![Cell::new(k, bold), Cell::new(v, Default::default())]);
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
        flatten(value, &mut flat, None, true)
            .map_err(|e| anyhow::Error::msg(e.description().to_string()))?;
        let obj = flat.as_object().expect("must be a object");
        let mut cells = vec![];
        for field in &self.field_names {
            let v = obj.get(field).unwrap_or(&Value::Null);
            cells.push(Cell::new(value_to_string(v).as_str(), Default::default()));
        }
        Ok(Row::new(cells))
    }
}
