// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `LaunchCreated`.
class LaunchCreatedEvent extends PinaBondingCurveEvent {
  const LaunchCreatedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.config,
    required this.creator,
    required this.baseMint,
    required this.totalSupply,
    required this.activationTime,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final Address config;
  final Address creator;
  final Address baseMint;
  final BigInt totalSupply;
  final BigInt activationTime;

  @override
  String get name => 'launchCreated';

  String toString() =>
      'LaunchCreatedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, config: ${config}, creator: ${creator}, baseMint: ${baseMint}, totalSupply: ${totalSupply}, activationTime: ${activationTime})';
}

/// The discriminator this event is emitted under.
const launchCreatedEventDiscriminator = 2;

/// The discriminator bytes as stored at offset zero.
const List<int> _launchCreatedEventDiscriminatorBytes = [2];

/// The migration version this event decodes.
const launchCreatedEventMigrationVersion = 0;

/// Exact current byte length of a `LaunchCreated` record, envelope included.
const launchCreatedEventSize = 146;

/// Decode one `LaunchCreated` record.
LaunchCreatedEvent decodeLaunchCreatedEvent(Uint8List data) {
  if (data.length != launchCreatedEventSize) {
    throw RangeError(
      'expected exactly ${launchCreatedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 2) {
    throw RangeError(
      'the provided bytes do not match the "LaunchCreated" event discriminator',
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
  final (v4, c4) = getAddressDecoder().read(data, cursor);
  cursor = c4;
  final (v5, c5) = getAddressDecoder().read(data, cursor);
  cursor = c5;
  final (v6, c6) = getU64Decoder().read(data, cursor);
  cursor = c6;
  final (v7, c7) = getI64Decoder().read(data, cursor);
  cursor = c7;

  return LaunchCreatedEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    config: v3,
    creator: v4,
    baseMint: v5,
    totalSupply: v6,
    activationTime: v7,
  );
}

/// A decoded `LaunchCreated` log record.
typedef DecodedLaunchCreatedEvent = LaunchCreatedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
LaunchCreatedEvent? parseLaunchCreatedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _launchCreatedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != launchCreatedEventMigrationVersion) {
    return null;
  }
  return decodeLaunchCreatedEvent(bytes);
}
