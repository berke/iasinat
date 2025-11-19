use super::*;

pub fn mphr_times(mphr:&Mphr,nline:usize)->(f64,f64,f64) {
    let t_start = mphr.sensing_start.to_unix();
    let t_end = mphr.sensing_end.to_unix();
    let delta_t = (t_start - t_end)/(nline - 1).max(1) as f64;
    (t_start,t_end,delta_t)
}
