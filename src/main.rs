// Rust code to perform numerical integration using multi-threading
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{env, io};

// sub-routine for computing integral of the function (sin x/ x) using the quadrature method
fn compute_quadrature(
    id: usize,
    samples_per_thread: usize,
    lower_limit: i32,
    upper_limit: i32,
    avg_arr: Arc<Mutex<Vec<f64>>>,
) {
    //random number generator
    let mut rng = rand::thread_rng();

    let mut sum_area = 0.0;
    for _i in 0..samples_per_thread {
        let x = rng.gen_range(lower_limit as f64..upper_limit as f64);
        let h = x.sin() / x;
        let area = h * (upper_limit as f64 - lower_limit as f64);
        sum_area += area;
    }

    let mut locked_avg_arr = avg_arr.lock().unwrap();
    locked_avg_arr[id] = sum_area / samples_per_thread as f64;
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut wrtr = csv::Writer::from_writer(io::stdout());
    if args.len() < 5 {
        eprintln!("**Invalid Input Format**\nReqd format : integrate a b n n_threads\n");
        std::process::exit(0);
    } else {
        let lower_limit: i32 = args[1].parse().unwrap();
        let upper_limit: i32 = args[2].parse().unwrap();
        let num_samples: usize = args[3].parse().unwrap();
        let n_threads: usize = args[4].parse().unwrap();

        assert!(
            lower_limit < upper_limit,
            "Invalid Limits! a greater than b!!!"
        );

        let mut do_profile: bool = false;

        if args.len() == 6 {
            match args[5].as_str() {
                "profile" => {
                    do_profile = true;
                }
                _ => {
                    eprintln!("Wrong format!!!");
                    std::process::exit(0);
                }
            }
        }

        // println!("{do_profile}")

        if lower_limit == 0 || upper_limit == 0 || n_threads == 0 {
            eprintln!("Invalid Value of a or b: zero not allowed");
            std::process::exit(0);
        }

        /*
           if the arg "profile" is passed while executing this code then the code would be timed incrementally for
           threads 1 to n_threads. Otherwise, the code is just timed for n_threads.
        */
        let mut curr_total_threads: usize = 1;
        if !do_profile {
            curr_total_threads = n_threads;
        }

        let mut sequential_timing: u128 = 0;
        wrtr.write_record(&["num_threads", "time", "speedup", "efficiency", "integral"])
            .expect("error in writing to file");

        for curr_total_threads in curr_total_threads..=n_threads {
            let avg_arr = vec![0.0; curr_total_threads];
            let mutex_avg_arr = Mutex::new(avg_arr);
            let arc_avg_arr = Arc::new(mutex_avg_arr); // using it since it is thread-safe

            // calculate samples to be distributed over all threads
            let samples_per_thread = num_samples / curr_total_threads;

            // println!("Samples per thread: {}", samples_per_thread);

            //vector to store all thread handles
            let mut thread_handle_arr = vec![];

            //start time
            let start = Instant::now();

            // spawning worker threads
            for thread_idx in 0..curr_total_threads {
                let arc_avg_arr_c = arc_avg_arr.clone();
                thread_handle_arr.push(std::thread::spawn(move || {
                    compute_quadrature(
                        thread_idx,
                        samples_per_thread,
                        lower_limit,
                        upper_limit,
                        arc_avg_arr_c,
                    );
                }));
            }

            //joining all worker threads
            for handle in thread_handle_arr {
                handle.join().unwrap();
            }

            // time taken to compute integral
            let elapsed_time = start.elapsed().as_micros();

            let mut final_ans = 0.0;
            for i in 0..curr_total_threads {
                final_ans += arc_avg_arr.lock().unwrap()[i] / curr_total_threads as f64;
            }

            if curr_total_threads == 1 {
                sequential_timing = elapsed_time;
            }

            let speedup = sequential_timing as f64 / elapsed_time as f64;
            let efficiency = speedup / curr_total_threads as f64;

            wrtr.write_record(&[
                curr_total_threads.to_string(),
                elapsed_time.to_string(),
                speedup.to_string(),
                efficiency.to_string(),
                final_ans.to_string(),
            ])
            .expect("error in writing to file");

            // println!("Final ans: {} in {} microseconds", final_ans, elapsed_time);
        }
    }
    wrtr.flush().expect("error in flushing file!");
}
