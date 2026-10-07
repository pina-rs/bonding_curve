// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `Claimed`.
class ClaimedEvent extends PinaBondingCurveEvent {
  const ClaimedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.claimant,
    required this.kind,
    required this.amount,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final Address claimant;
  final int kind;
  final BigInt amount;

  @override
  String get name => 'claimed';

  String toString() =>
      'ClaimedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, claimant: ${claimant}, kind: ${kind}, amount: ${amount})';
}

/// The discriminator this event is emitted under.
const claimedEventDiscriminator = 6;

/// The discriminator bytes as stored at offset zero.
const List<int> _claimedEventDiscriminatorBytes = [6];

/// The migration version this event decodes.
const claimedEventMigrationVersion = 0;

/// Exact current byte length of a `Claimed` record, envelope included.
const claimedEventSize = 75;

/// Decode one `Claimed` record.
ClaimedEvent decodeClaimedEvent(Uint8List data) {
  if (data.length != claimedEventSize) {
    throw RangeError(
      'expected exactly ${claimedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 6) {
    throw RangeError(
      'the provided bytes do not match the "Claimed" event discriminator',
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
  final (v4, c4) = getU8Decoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getU64Decoder().read(data, cursor);
  cursor = c5;

  return ClaimedEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    claimant: v3,
    kind: v4,
    amount: v5,
  );
}

/// A decoded `Claimed` log record.
typedef DecodedClaimedEvent = ClaimedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
ClaimedEvent? parseClaimedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _claimedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != claimedEventMigrationVersion) {
    return null;
  }
  return decodeClaimedEvent(bytes);
}
