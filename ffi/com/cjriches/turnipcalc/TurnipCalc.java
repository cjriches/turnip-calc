/*

This file is the Java end of the JNI wrapper for libturnipcalc.
It is intended to be directly dropped into any Java codebase that needs it;
just remove the main function and you're good to go.
The license text is already included below, so you can do anything with this
code except remove said license text.



MIT License

Copyright (c) 2021 Chris Riches

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

package com.cjriches.turnipcalc;

public class TurnipCalc {
    static {
        System.loadLibrary("turnipcalc");
    }

    public static final byte DECREASING = 1;
    public static final byte RANDOM = 2;
    public static final byte SMALL_SPIKE = 3;
    public static final byte LARGE_SPIKE = 4;

    public static class PatternResult {
        public final byte pattern;
        public final double probability;

        public PatternResult(final byte pattern, final double probability) {
            this.pattern = pattern;
            this.probability = probability;
        }
    }

    public static class CalcResult {
        public final boolean success;
        public final PatternResult[] results;

        public CalcResult(final boolean success, final PatternResult[] results) {
            this.success = success;
            this.results = results;
        }
    }

    public static native CalcResult run(byte prev_pattern, int base_price, int[] prices);

    public static void main(final String[] args) {
        final CalcResult result = run(SMALL_SPIKE, 95, new int[]{85, 81});
        System.out.println("Success: " + result.success);
        for (final PatternResult pattern : result.results) {
            System.out.print(pattern.pattern);
            System.out.println(" " + pattern.probability);
        }
    }
}
