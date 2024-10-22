#![feature(iter_map_windows)]
use ndarray::parallel::prelude::IndexedParallelIterator;
use ndarray::parallel::prelude::IntoParallelIterator;
use ndarray::parallel::prelude::ParallelIterator;
use numpy::{PyArray1, PyArray2, PyReadonlyArrayDyn};
use numpy::ndarray::{ArrayView1};
use pyo3::prelude::*;



#[pymodule]
fn splits_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(split_string, m)?)?;
    m.add_function(wrap_pyfunction!(split_strings, m)?)?;
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