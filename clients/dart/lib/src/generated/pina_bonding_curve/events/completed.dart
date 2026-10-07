// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `Completed`.
class CompletedEvent extends PinaBondingCurveEvent {
  const CompletedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.quoteReserve,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final BigInt quoteReserve;

  @override
  String get name => 'completed';

  String toString() =>
      'CompletedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, quoteReserve: ${quoteReserve})';
}

/// The discriminator this event is emitted under.
const completedEventDiscriminator = 4;

/// The discriminator bytes as stored at offset zero.
const List<int> _completedEventDiscriminatorBytes = [4];

/// The migration version this event decodes.
const completedEventMigrationVersion = 0;

/// Exact current byte length of a `Completed` record, envelope included.
const completedEventSize = 42;

/// Decode one `Completed` record.
CompletedEvent decodeCompletedEvent(Uint8List data) {
  if (data.length != completedEventSize) {
    throw RangeError(
      'expected exactly ${completedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 4) {
    throw RangeError(
      'the provided bytes do not match the "Completed" event discriminator',
    );
  }
  final (v1, c1) = getU8Decoder().read(data, cursor);
  cursor = c1;
  if (v1 != 0) {
    throw RangeError(
      v1 < 0
          ? 'event migration version mismatch: expected 0, received $v1 (decode it with the event for that version)'
          : 'event migration version mismatch: expected 0, received $v1 (the log was written by a newer program; upgrade this client)',
    );
  }
  final (v2, c2) = getAddressDecoder().read(data, cursor);
  cursor = c2;
  final (v3, c3) = getU64Decoder().read(data, cursor);
  cursor = c3;

  return CompletedEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    quoteReserve: v3,
  );
}

/// A decoded `Completed` log record.
typedef DecodedCompletedEvent = CompletedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
CompletedEvent? parseCompletedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _completedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != completedEventMigrationVersion) {
    return null;
  }
  return decodeCompletedEvent(bytes);
}
