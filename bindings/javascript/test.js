#!/usr/bin/env node

import { generateMorseTiming, generateMorseAudio, playMorseAudio, ready, AUDIO_MODE, WAVEFORM_TYPE } from './morse.js';

// Simple test framework
let testsRun = 0;
let testsPassed = 0;

function test(name, testFn) {
  testsRun++;
  process.stdout.write(`Running ${name}... `);
  try {
    if (testFn()) {
      testsPassed++;
      console.log('PASS');
    } else {
      console.log('FAIL');
    }
  } catch (error) {
    console.log(`FAIL - ${error.message}`);
  }
}

// Wait for WASM module to load
await ready;

console.log('JavaScript Binding Tests');
console.log('========================\n');

// Test basic timing generation
test('basic_timing', () => {
  const result = generateMorseTiming({ text: 'E' });
  return result.length === 1 && result[0].type === 'dot';
});

// Test timing with multiple characters
test('multi_character_timing', () => {
  const result = generateMorseTiming({ text: 'SOS' });
  return result.length > 5 && result.some(e => e.type === 'dot') && result.some(e => e.type === 'dash');
});

// Test WPM parameter
test('wpm_parameter', () => {
  const fast = generateMorseTiming({ text: 'E', wpm: 40 });
  const slow = generateMorseTiming({ text: 'E', wpm: 10 });
  return fast[0].duration_seconds < slow[0].duration_seconds;
});

// Test audio generation
test('audio_generation', () => {
  const result = generateMorseAudio({ text: 'E', sampleRate: 8000 });
  return result.audioData.length > 0 &&
         result.sampleRate === 8000 &&
         result.duration > 0 &&
         result.audioData.some(sample => sample !== 0);
});

// Test audio parameters
test('audio_parameters', () => {
  const result1 = generateMorseAudio({ text: 'E', frequency: 880 });
  const result2 = generateMorseAudio({ text: 'E', frequency: 440 });
  // Different frequencies should produce different audio (simple check)
  return result1.audioData.length > 0 && result2.audioData.length > 0;
});

// Test input validation
test('input_validation', () => {
  try {
    generateMorseTiming({ text: 'E', wpm: 0 });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('WPM');
  }
});

// Test invalid text input
test('invalid_text', () => {
  try {
    generateMorseTiming({ text: null });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('Invalid text');
  }
});

// Test prosign syntax
test('prosign_syntax', () => {
  const result = generateMorseTiming({ text: '[SOS]' });
  return result.length > 0 && result.some(e => e.type === 'gap');
});

// Test larger text
test('large_text', () => {
  const result = generateMorseAudio({
    text: 'THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG',
    sampleRate: 8000
  });
  return result.audioData.length > 1000 && result.duration > 1.0;
});

// Test Radio mode audio generation
test('radio_mode_audio', () => {
  const result = generateMorseAudio({
    text: 'SOS',
    audioMode: AUDIO_MODE.RADIO,
    frequency: 600,
    backgroundStaticLevel: 0.1,
    sampleRate: 8000
  });
  return result.audioData.length > 0 && result.duration > 0;
});

// Test Telegraph mode audio generation
test('telegraph_mode_audio', () => {
  const result = generateMorseAudio({
    text: 'SOS',
    audioMode: AUDIO_MODE.TELEGRAPH,
    clickSharpness: 0.7,
    resonanceFreq: 1000,
    decayRate: 15.0,
    mechanicalNoise: 0.2,
    sampleRate: 8000
  });
  return result.audioData.length > 0 && result.duration > 0;
});

// Test audio mode validation
test('audio_mode_validation', () => {
  try {
    generateMorseAudio({
      text: 'SOS',
      audioMode: 2, // Invalid mode
      sampleRate: 8000
    });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('Audio mode');
  }
});

// Test Radio mode parameter validation
test('radio_parameter_validation', () => {
  try {
    generateMorseAudio({
      text: 'SOS',
      audioMode: AUDIO_MODE.RADIO,
      frequency: -100, // Invalid frequency
      sampleRate: 8000
    });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('Frequency');
  }
});

// Test Telegraph mode parameter validation
test('telegraph_parameter_validation', () => {
  try {
    generateMorseAudio({
      text: 'SOS',
      audioMode: AUDIO_MODE.TELEGRAPH,
      clickSharpness: 1.5, // Invalid (> 1.0)
      sampleRate: 8000
    });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('Click sharpness');
  }
});

// Test Radio mode waveform types
test('radio_waveform_types', () => {
  const waveforms = [WAVEFORM_TYPE.SINE, WAVEFORM_TYPE.SQUARE, WAVEFORM_TYPE.SAWTOOTH, WAVEFORM_TYPE.TRIANGLE];

  for (const waveformType of waveforms) {
    const result = generateMorseAudio({
      text: 'E',
      audioMode: AUDIO_MODE.RADIO,
      waveformType,
      sampleRate: 8000
    });
    if (result.audioData.length === 0 || result.duration === 0) {
      return false;
    }
  }
  return true;
});

// Test waveform type validation
test('waveform_type_validation', () => {
  try {
    generateMorseAudio({
      text: 'SOS',
      audioMode: AUDIO_MODE.RADIO,
      waveformType: 4, // Invalid (> 3)
      sampleRate: 8000
    });
    return false; // Should have thrown
  } catch (error) {
    return error.message.includes('Waveform type');
  }
});

// Test background static
test('background_static', () => {
  const withStatic = generateMorseAudio({
    text: 'E',
    audioMode: AUDIO_MODE.RADIO,
    backgroundStaticLevel: 0.3,
    sampleRate: 8000
  });

  const withoutStatic = generateMorseAudio({
    text: 'E',
    audioMode: AUDIO_MODE.RADIO,
    backgroundStaticLevel: 0.0,
    sampleRate: 8000
  });

  // Audio with static should be different from without static
  // Check that they have different energy levels
  let staticSum = 0, cleanSum = 0;
  for (let i = 0; i < Math.min(withStatic.audioData.length, withoutStatic.audioData.length); i++) {
    staticSum += Math.abs(withStatic.audioData[i]);
    cleanSum += Math.abs(withoutStatic.audioData[i]);
  }

  // Static version should have higher average energy
  return staticSum > cleanSum;
});

console.log(`\nResults: ${testsPassed}/${testsRun} tests passed`);

if (testsPassed === testsRun) {
  console.log('All tests PASSED!');
  process.exit(0);
} else {
  console.log('Some tests FAILED!');
  process.exit(1);
}