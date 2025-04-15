pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn sqrt(x: u64) -> u64{
    if x <=1{
        return x;
    }
    let mut left= 0;
    let mut right = x;
    let mut result =0;
    while left <= right{
        let mid = (left +right)/2;
        if mid *mid <=x{
            left = mid+1;
            result = mid;
        } else{
            right = mid-1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
