use chrono::DateTime;
use chrono::Utc;
pub use cron::Schedule;

/// A schedulable `Job`.
pub struct Job<'a> {
    schedule: Schedule,
    run: Box<(FnMut() -> ()) + 'a>,
    last_tick: Option<DateTime<Utc>>,
    limit_missed_runs: usize,
    job_id: String,
}

impl<'a> Job<'a> {
    /// Create a new job.
    ///
    /// ```rust,ignore
    /// // Run at second 0 of the 15th minute of the 6th, 8th, and 10th hour
    /// // of any day in March and June that is a Friday of the year 2017.
    /// let s: Schedule = "0 15 6,8,10 * Mar,Jun Fri 2017".into().unwrap();
    /// Job::new(s, || println!("I have a complex schedule...") );
    /// ```
    pub fn new<T>(schedule: Schedule, run: T) -> Job<'a>
        where T: 'a,
              T: FnMut() -> ()
    {
        Job {
            schedule,
            run: Box::new(run),
            last_tick: None,
            limit_missed_runs: 1,
            job_id: "test".to_owned(),
        }
    }

    fn tick(&mut self) {
        let now = Utc::now();
        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }
        if self.limit_missed_runs > 0 {
            for event in self.schedule.after(&self.last_tick.unwrap()).take(self.limit_missed_runs) {
                if event > now { break; }
                (self.run)();
            }
        }
        else {
            for event in self.schedule.after(&self.last_tick.unwrap()) {
                if event > now { break; }
                (self.run)();
            }
        }

        self.last_tick = Some(now);
    }

    /// Set the limit for missed jobs in the case of delayed runs. Setting to 0 means unlimited.
    ///
    /// ```rust,ignore
    /// let mut job = Job::new("0/1 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 1 seconds!");
    /// });
    /// job.limit_missed_runs(99);
    /// ```
    pub fn limit_missed_runs(&mut self, limit: usize) {
        self.limit_missed_runs = limit;
    }
}

#[derive(Default)]
/// The JobScheduler contains and executes the scheduled jobs.
pub struct JobScheduler<'a> {
    jobs: Vec<Job<'a>>,
}

impl<'a> JobScheduler<'a> {
    /// Create a new `JobScheduler`.
    pub fn new() -> JobScheduler<'a> {
        JobScheduler { jobs: Vec::new() }
    }

    /// Add a job to the `JobScheduler`
    ///
    /// ```rust,ignore
    /// let mut sched = JobScheduler::new();
    /// sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// ```
    pub fn add(&mut self, job: Job<'a>) -> String {
        let job_id = job.job_id.clone();
        self.jobs.push(job);

        job_id
    }

    /// Remove a job from the `JobScheduler`
    ///
    /// ```rust,ignore
    /// let mut sched = JobScheduler::new();
    /// let job_id = sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// sched.remove(job_id);
    /// ```
    pub fn remove(&mut self, job_id: String) -> bool {
        let mut found_index = None;
        for (i, job) in self.jobs.iter().enumerate() {
            if job.job_id == job_id {
                found_index = Some(i);
                break;
            }
        }

        if found_index.is_some() {
            self.jobs.remove(found_index.unwrap());
        }

        found_index.is_some()
    }

    /// The `tick` method increments time for the JobScheduler and executes
    /// any pending jobs. It is recommended to sleep for at least 500
    /// milliseconds between invocations of this method.
    ///
    /// ```rust,ignore
    /// loop {
    ///     sched.tick();
    ///     std::thread::sleep(Duration::from_millis(500));
    /// }
    /// ```
    pub fn tick(&mut self) {
        for mut job in &mut self.jobs {
            job.tick();
        }
    }
}