use jni::JNIEnv;
use jni::objects::{JClass, JValue, ReleaseMode};
use jni::sys::{jboolean, jbyte, jdouble, jint, jintArray, jobject, jsize};
use std::ptr;

use super::*;

// Class paths
const PATTERN_RESULT: &str = "com/cjriches/turnipcalc/TurnipCalc$PatternResult";
const CALC_RESULT: &str = "com/cjriches/turnipcalc/TurnipCalc$CalcResult";
// Constructor signatures
const PATTERN_RESULT_CTOR: &str = "(BD)V";
const CALC_RESULT_CTOR: &str = "(Z[Lcom/cjriches/turnipcalc/TurnipCalc$PatternResult;)V";

/// This can be referenced in Java code by a:
/// * method called `run`
/// * in a class called `TurnipCalc`
/// * in a package called `com.cjriches.turnipcalc`.
///
/// Such a class is provided at [../com/cjriches/turnipcalc/TurnipCalc.java].
/// Just remove the main function and plug it into your Java application.
/// ```
#[no_mangle]
pub unsafe extern "system" fn Java_com_cjriches_turnipcalc_TurnipCalc_run(
    env: JNIEnv, _: JClass, prev_pattern: jbyte, base_price: jint, prices: jintArray)
    -> jobject
{
    // Convert arguments.
    let prev_pattern_conv = prev_pattern as u8;
    let base_price_conv = base_price as u32;
    let prices_arr = env.get_int_array_elements(prices, ReleaseMode::NoCopyBack)
        .expect("Failed to read prices array");  // Must keep a reference or the array gets freed underneath us.
    let prices_conv = prices_arr.as_ptr() as *const u32;
    let num_prices = env.get_array_length(prices)
        .expect("Failed to read prices array length") as usize;

    // Run implementation.
    let calc_result = turnip_calc(prev_pattern_conv, base_price_conv, prices_conv, num_prices);

    // Construct return value.
    let pattern_results = env.new_object_array(calc_result.num as jsize,
                                               PATTERN_RESULT, ptr::null_mut())
        .expect("Failed to create result array");
    for i in 0..calc_result.num {
        let raw_pr = ptr::read(calc_result.results.offset(i as isize));
        let pattern = JValue::Byte(raw_pr.pattern as jbyte);
        let probability = JValue::Double(raw_pr.probability as jdouble);
        let pattern_result = env.new_object(PATTERN_RESULT, PATTERN_RESULT_CTOR, &[pattern, probability])
            .expect("Failed to create PatternResult");
        env.set_object_array_element(pattern_results, i as jsize, pattern_result)
            .expect("Failed to insert into result array");
    }
    let success = JValue::Bool(calc_result.success as jboolean);
    let results = JValue::Object(pattern_results.into());
    let calc_result_java = env.new_object(CALC_RESULT, CALC_RESULT_CTOR, &[success, results])
        .expect("Failed to create CalcResult");

    // Free memory.
    free_result(calc_result);

    return calc_result_java.into_inner();
}
