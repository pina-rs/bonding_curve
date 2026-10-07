// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `Graduated`.
class GraduatedEvent extends PinaBondingCurveEvent {
  const GraduatedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.pool,
    required this.poolBase,
    required this.poolQuote,
    required this.migrationFee,
    required this.burnedBase,
    required this.burnedLp,
    required this.creatorLp,
    required this.partnerLp,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final Address pool;
  final BigInt poolBase;
  final BigInt poolQuote;
  final BigInt migrationFee;
  final BigInt burnedBase;
  final BigInt burnedLp;
  final BigInt creatorLp;
  final BigInt partnerLp;

  @override
  String get name => 'graduated';

  String toString() =>
      'GraduatedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, pool: ${pool}, poolBase: ${poolBase}, poolQuote: ${poolQuote}, migrationFee: ${migrationFee}, burnedBase: ${burnedBase}, burnedLp: ${burnedLp}, creatorLp: ${creatorLp}, partnerLp: ${partnerLp})';
}

/// The discriminator this event is emitted under.
const graduatedEventDiscriminator = 5;

/// The discriminator bytes as stored at offset zero.
const List<int> _graduatedEventDiscriminatorBytes = [5];

/// The migration version this event decodes.
const graduatedEventMigrationVersion = 0;

/// Exact current byte length of a `Graduated` record, envelope included.
const graduatedEventSize = 122;

/// Decode one `Graduated` record.
GraduatedEvent decodeGraduatedEvent(Uint8List data) {
  if (data.length != graduatedEventSize) {
    throw RangeError(
      'expected exactly ${graduatedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 5) {
    throw RangeError(
      'the provided bytes do not match the "Graduated" event discriminator',
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
  final (v3, c3) = getAddressDecoder().read(data, cursor);
  cursor = c3;
  final (v4, c4) = getU64Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU64Decoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU64Decoder().read(data, cursor);
  cursor = c6;
  final (v7, c7) = getU64Decoder().read(data, cursor);
  cursor = c7;
  final (v8, c8) = getU64Decoder().read(data, cursor);
  cursor = c8;
  final (v9, c9) = getU64Decoder().read(data, cursor);
  cursor = c9;
  final (v10, c10) = getU64Decoder().read(data, cursor);
  cursor = c10;

  return GraduatedEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    pool: v3,
    poolBase: v4,
    poolQuote: v5,
    migrationFee: v6,
    burnedBase: v7,
    burnedLp: v8,
    creatorLp: v9,
    partnerLp: v10,
  );
}

/// A decoded `Graduated` log record.
typedef DecodedGraduatedEvent = GraduatedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
GraduatedEvent? parseGraduatedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _graduatedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != graduatedEventMigrationVersion) {
    return null;
  }
  return decodeGraduatedEvent(bytes);
}
