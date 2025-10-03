#!/usr/bin/env node

import {
  generateMorseTiming,
  generateMorseAudio,
  playMorseAudio,
  interpretMorseSignals,
  MorseAudioMode,
  MorseWaveformType,
} from "./morse.js";

// Simple test framework
let testsRun = 0;
let testsPassed = 0;

function test(name, testFn) {
  testsRun++;
  process.stdout.write(`Running ${name}... `);
  try {
    if (testFn()) {
      testsPassed++;
      console.log("PASS");
    } else {
      console.log("FAIL");
    }
  } catch (error) {
    console.log(`FAIL - ${error.message}`);
  }
}

console.log("JavaScript Rust Binding Tests");
console.log("==============================\n");

// Test basic timing generation
test("basic_timing", () => {
  const result = generateMorseTiming({ text: "E" });
  return result.length === 1 && result[0].type === "dot";
});

// Test timing with multiple characters
test("multi_character_timing", () => {
  const result = generateMorseTiming({ text: "SOS" });
  return (
    result.length > 5 &&
    result.some((e) => e.type === "dot") &&
    result.some((e) => e.type === "dash")
  );
});

// Test WPM parameter
test("wpm_parameter", () => {
  const fast = generateMorseTiming({ text: "E", wpm: 40 });
  const slow = generateMorseTiming({ text: "E", wpm: 10 });
  return fast[0].duration_seconds < slow[0].duration_seconds;
});

// Test audio generation
test("audio_generation", () => {
  const result = generateMorseAudio({ text: "E" });
  return (
    result &&
    result.audioData &&
    result.audioData.length > 0 &&
    result.sampleRate > 0
  );
});

// Test radio mode audio
test("radio_mode", () => {
  const result = generateMorseAudio({
    text: "E",
    audioMode: MorseAudioMode.Radio,
    frequency: 600,
    waveformType: MorseWaveformType.Sine,
  });
  return result && result.audioData && result.audioData.length > 0;
});

// Test telegraph mode audio
test("telegraph_mode", () => {
  const result = generateMorseAudio({
    text: "E",
    audioMode: MorseAudioMode.Telegraph,
    clickSharpness: 0.7,
    resonanceFreq: 800,
  });
  return result && result.audioData && result.audioData.length > 0;
});

// Test different waveforms
test("different_waveforms", () => {
  const sine = generateMorseAudio({
    text: "E",
    waveformType: MorseWaveformType.Sine,
  });
  const square = generateMorseAudio({
    text: "E",
    waveformType: MorseWaveformType.Square,
  });
  const sawtooth = generateMorseAudio({
    text: "E",
    waveformType: MorseWaveformType.Sawtooth,
  });
  const triangle = generateMorseAudio({
    text: "E",
    waveformType: MorseWaveformType.Triangle,
  });

  return (
    sine.audioData.length > 0 &&
    square.audioData.length > 0 &&
    sawtooth.audioData.length > 0 &&
    triangle.audioData.length > 0
  );
});

// Test humanization
test("humanization", () => {
  const normal = generateMorseTiming({ text: "E", randomSeed: 12345 });
  const humanized = generateMorseTiming({
    text: "E",
    humanizationFactor: 0.5,
    randomSeed: 12345,
  });

  // With same seed, should get same result since humanization is deterministic
  return normal.length === humanized.length;
});

// Test prosign parsing
test("prosign_brackets", () => {
  const result = generateMorseTiming({ text: "[SOS]" });
  return (
    result.length > 0 &&
    result.some((e) => e.type === "dot") &&
    result.some((e) => e.type === "dash")
  );
});

// Test validation
test("parameter_validation", () => {
  try {
    generateMorseTiming({ text: "", wpm: -1 });
    return false; // Should have thrown
  } catch (error) {
    return (
      error.message.includes("WPM") ||
      error.message.includes("Invalid text input")
    );
  }
});

// Test interpretation (should fail as not implemented)
test("interpretation_not_implemented", () => {
  try {
    interpretMorseSignals({ signals: [{ on: true, seconds: 0.1 }] });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes("not implemented");
  }
});

// Summary
console.log(`\nTest Results: ${testsPassed}/${testsRun} tests passed`);
if (testsPassed === testsRun) {
  console.log("All tests passed! ✓");
  process.exit(0);
} else {
  console.log("Some tests failed! ✗");
  process.exit(1);
}
