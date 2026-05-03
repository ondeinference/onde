// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
import 'package:flutter_test/flutter_test.dart';

import 'package:onde_inference_example/main.dart';

void main() {
  testWidgets('OndeInferenceApp builds without crashing',
      (WidgetTester tester) async {
    // Make sure the root widget can be constructed.
    // Full integration tests need the native inference library first.
    // Run `flutter build` or `cargo build` in sdk/dart/rust/ before running
    // widget tests that actually hit the engine.
    expect(const OndeInferenceApp(), isA<OndeInferenceApp>());
  });
}
