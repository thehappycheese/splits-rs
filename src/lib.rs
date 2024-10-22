#![feature(iter_map_windows)]
use std::sync::Arc;

use arrow::array::make_array;
use arrow::array::Array;
use arrow::array::ArrayData;
use arrow::array::StringArray;
use arrow::array::StringBuilder;
use arrow::array::StringViewArray;
use arrow::array::StructArray;
use arrow::array::UInt64Builder;
use arrow::datatypes::DataType;
use arrow::datatypes::Field;
use arrow::pyarrow::PyArrowType;
use ndarray::parallel::prelude::IndexedParallelIterator;
use ndarray::parallel::prelude::IntoParallelIterator;
use ndarray::parallel::prelude::ParallelIterator;
use numpy::PyReadonlyArrayDyn;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;



#[pymodule]
fn splits_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(split_string, m)?)?;
    m.add_function(wrap_pyfunction!(split_strings, m)?)?;
    m.add_function(wrap_pyfunction!(split_strings_arrow, m)?)?;
    Ok(())
}

#[pyfunction]
fn split_string(input:&str)->PyResult<Vec<String>>{
    let ws = input.split_whitespace();
    let result:Vec<String> = ws.clone().map_windows(|[a,b,c]| format!("{a} {b} {c}")).chain(
        ws.clone().map_windows(|[a,b]| format!("{a} {b}"))
    ).chain(
        ws.clone().map(|a|a.to_owned())
    ).collect();
    return Ok(result);
}

#[pyfunction]
fn split_strings<'py>(input: PyReadonlyArrayDyn<'py, PyObject>, py:Python<'_>) -> PyResult<Vec<(usize, String)>> {
    // First, do single-threaded extraction of the strings
    let extracted_strings: Vec<String> = input
        .as_array()
        .iter()
        .filter_map(|item| {
            // Try to extract each item as a String
            match item.extract::<String>(py) {
                Ok(s) => Some(s),  // If successful, include the string
                Err(_) => None,    // If not a string, skip it
            }
        })
        .collect();

    // Now that we have a Vec<String>, process it in parallel
    let result: Vec<(usize, String)> = extracted_strings
        .into_par_iter() // Parallel iterator
        .enumerate()
        .flat_map(|(i, s)| {
            // Call split_string on each extracted string
            match split_string(&s) {
                Ok(split_result) => split_result.into_iter().map(|s| (i, s)).collect(), // Return the result of split_string
                Err(_) => vec![],                 // Handle the error (empty result)
            }
        })
        .collect();

    Ok(result)
}

#[pyfunction]
fn split_strings_arrow<'py>(input: PyArrowType<ArrayData>) -> PyResult<(PyArrowType<ArrayData>,PyArrowType<ArrayData>)> {
    // First, do single-threaded extraction of the strings
    let input = input.0; // Extract from PyArrowType wrapper
    let array: Arc<dyn Array> = make_array(input); // Convert ArrayData to ArrayRef
    let array: &StringArray = array.as_any().downcast_ref()
        .ok_or_else(|| PyValueError::new_err("Could not cast to string array"))?;

    

    let mut result:(UInt64Builder, StringBuilder) = array
        .iter()
        .enumerate()
        .fold(
            (UInt64Builder::new(), StringBuilder::new()),
            |(mut v_a, mut v_b),(i, s)| match s {
            Some(s)=>{
                let ws = s.split_whitespace();
                let _:Vec<_> = ws.clone().map_windows(|[a,b,c]| {
                    v_a.append_value(i as u64);
                    v_b.append_value(format!("{a} {b} {c}"));
                }).collect();
                let _:Vec<_> = ws.clone().map_windows(|[a,b]| {
                    v_a.append_value(i as u64);
                    v_b.append_value(format!("{a} {b}"));
                }).collect();

                let _:Vec<_> = ws.clone().map(|a| {
                    v_a.append_value(i as u64);
                    v_b.append_value(a.to_owned());
                }).collect();

                (v_a, v_b)
            }
            None=>(v_a, v_b)
        });
    Ok((
        PyArrowType(result.0.finish().into_data()),
        PyArrowType(result.1.finish().into_data())
    ))
}
