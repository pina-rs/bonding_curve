// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `DustSwept`.
class DustSweptEvent extends PinaBondingCurveEvent {
  const DustSweptEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.amount,
    required this.creatorFee,
    required this.partnerFee,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final BigInt amount;
  final BigInt creatorFee;
  final BigInt partnerFee;

  @override
  String get name => 'dustSwept';

  String toString() =>
      'DustSweptEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, amount: ${amount}, creatorFee: ${creatorFee}, partnerFee: ${partnerFee})';
}

/// The discriminator this event is emitted under.
const dustSweptEventDiscriminator = 8;

/// The discriminator bytes as stored at offset zero.
const List<int> _dustSweptEventDiscriminatorBytes = [8];

/// The migration version this event decodes.
const dustSweptEventMigrationVersion = 0;

/// Exact current byte length of a `DustSwept` record, envelope included.
const dustSweptEventSize = 58;

/// Decode one `DustSwept` record.
DustSweptEvent decodeDustSweptEvent(Uint8List data) {
  if (data.length != dustSweptEventSize) {
    throw RangeError(
      'expected exactly ${dustSweptEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 8) {
    throw RangeError(
      'the provided bytes do not match the "DustSwept" event discriminator',
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
  final (v4, c4) = getU64Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU64Decoder().read(data, cursor);
  cursor = c5;

  return DustSweptEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    amount: v3,
    creatorFee: v4,
    partnerFee: v5,
  );
}

/// A decoded `DustSwept` log record.
typedef DecodedDustSweptEvent = DustSweptEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
DustSweptEvent? parseDustSweptEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _dustSweptEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != dustSweptEventMigrationVersion) {
    return null;
  }
  return decodeDustSweptEvent(bytes);
}
