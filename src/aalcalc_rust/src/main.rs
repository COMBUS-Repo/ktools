//! This tool is responsible for calculating the average annual losses.
//!
//! # Running
//! You can run the program with the following command:
//! ```bash
//! aalcalc -k summary_aal
//! ```
//!
mod data_access_layer;
mod processes;
mod collections;

use data_access_layer::occurrence::{OccurrenceData, OccurrenceFileHandle};
use data_access_layer::period_weights::{PeriodWeights, PeriodWeightsHandle};
use data_access_layer::traits::load_period_weights::ReadPeriodWeights;
use data_access_layer::summary::get_summaries_from_data;
use collections::summary_statistics::SummaryStatistics;
use processes::get_all_binary_file_paths;
use crate::data_access_layer::summary_loader::SummaryLoaderHandle;
use crate::data_access_layer::traits::load_occurrence::ReadOccurrences;

use std::collections::HashMap;
use std::path::Path;

use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    k: String,
}


fn main() {
    let args = Args::parse();

    let pwd = std::env::current_dir().unwrap().into_os_string().into_string().unwrap();
    let occurrence_path = format!("{}/input/occurrence.bin", pwd);
    let period_weights_path = format!("{}/input/periods.bin", pwd);

    // get data around the occurrences
    let mut occ_data_handle = OccurrenceFileHandle::new(occurrence_path).unwrap();
    let number_of_periods = &occ_data_handle.get_meta_data().period_number;
    let mut occ_data = occ_data_handle.get_data();
    // let number_of_periods = occ_data_handle.get_meta_data().period_number;

    // define map for summary statistics
    let mut summary_map: HashMap<i32, SummaryStatistics> = HashMap::new();

    // get all files in directory
    let file_pattern = format!("{}/work/{}/*.bin", pwd, args.k);
    let files = get_all_binary_file_paths(file_pattern);

    // run the processes in parallel
    let _ = files.iter().map(|i| {

        // load all the data from the file
        let mut handle = SummaryLoaderHandle{};
        let summaries = get_summaries_from_data(i.clone(), &mut handle, &occ_data).unwrap();

        for summary in summaries {

            // extract the summary statistics if it exists, crease one if not
            let mut summary_statistics: &mut SummaryStatistics;
            match summary_map.get_mut(&summary.summary_id) {
                Some(statistics) => {
                    summary_statistics = statistics;
                },
                None => {
                    let statistics = SummaryStatistics::new(*&summary.summary_id.clone());
                    summary_map.insert(summary.summary_id.clone(), statistics);
                    summary_statistics = summary_map.get_mut(&summary.summary_id).unwrap();
                }
            }
            // update the summary statistics with the data from the summary
            summary_statistics.ingest_summary(summary);
        }
        return 1
    }).collect::<Vec<i32>>();

    // order the summary IDs for printing out
    let mut summary_ids: Vec<&i32> = summary_map.keys().collect();
    summary_ids.sort();
    
    let period_weights: Option<&Vec<f64>>;
    let period_weights_loader: Option<PeriodWeights>;

    if Path::new(&period_weights_path).exists() == true {
        println!("period weights are firing");
        let mut period_handle = PeriodWeightsHandle::new(period_weights_path).unwrap();
        period_weights_loader = Some(PeriodWeights{weights: period_handle.get_data()});
    }
    else {
        period_weights_loader = None;
    }
    match &period_weights_loader {
        Some(period_weights_reference) => {
            period_weights = Some(&period_weights_reference.weights);
        }, 
        None => {
            period_weights = None;
        }
    }

    println!("summary_id,type,mean,standard_deviation");

    // print out summary statistics
    for i in &summary_ids {
        let sum_stats = &summary_map.get(i).unwrap();
        sum_stats.print_type_one_stats(*number_of_periods, period_weights);
    }

    for i in &summary_ids {
        let sum_stats = &summary_map.get(i).unwrap();
        sum_stats.print_type_two_stats(*number_of_periods, period_weights);
    }
}
