// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';

import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `Traded`.
class TradedEvent extends PinaBondingCurveEvent {
  const TradedEvent({
    required this.discriminator,
    required this.migrationVersion,
    required this.launch,
    required this.trader,
    required this.isBuy,
    required this.baseAmount,
    required this.quoteAmount,
    required this.fee,
    required this.creatorFee,
    required this.sqrtPrice,
    required this.quoteReserve,
  });

  final int discriminator;
  final int migrationVersion;
  final Address launch;
  final Address trader;
  final int isBuy;
  final BigInt baseAmount;
  final BigInt quoteAmount;
  final BigInt fee;
  final BigInt creatorFee;
  final BigInt sqrtPrice;
  final BigInt quoteReserve;

  @override
  String get name => 'traded';

  String toString() =>
      'TradedEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, launch: ${launch}, trader: ${trader}, isBuy: ${isBuy}, baseAmount: ${baseAmount}, quoteAmount: ${quoteAmount}, fee: ${fee}, creatorFee: ${creatorFee}, sqrtPrice: ${sqrtPrice}, quoteReserve: ${quoteReserve})';
}

/// The discriminator this event is emitted under.
const tradedEventDiscriminator = 3;

/// The discriminator bytes as stored at offset zero.
const List<int> _tradedEventDiscriminatorBytes = [3];

/// The migration version this event decodes.
const tradedEventMigrationVersion = 0;

/// Exact current byte length of a `Traded` record, envelope included.
const tradedEventSize = 123;

/// Decode one `Traded` record.
TradedEvent decodeTradedEvent(Uint8List data) {
  if (data.length != tradedEventSize) {
    throw RangeError(
      'expected exactly ${tradedEventSize} bytes, received ${data.length}',
    );
  }
  var cursor = 0;
  final (v0, c0) = getU8Decoder().read(data, cursor);
  cursor = c0;
  if (v0 != 3) {
    throw RangeError(
      'the provided bytes do not match the "Traded" event discriminator',
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
  final (v6, c6) = getU64Decoder().read(data, cursor);
  cursor = c6;
  final (v7, c7) = getU64Decoder().read(data, cursor);
  cursor = c7;
  final (v8, c8) = getU64Decoder().read(data, cursor);
  cursor = c8;
  final (v9, c9) = getU128Decoder().read(data, cursor);
  cursor = c9;
  final (v10, c10) = getU64Decoder().read(data, cursor);
  cursor = c10;

  return TradedEvent(
    discriminator: v0,
    migrationVersion: v1,
    launch: v2,
    trader: v3,
    isBuy: v4,
    baseAmount: v5,
    quoteAmount: v6,
    fee: v7,
    creatorFee: v8,
    sqrtPrice: v9,
    quoteReserve: v10,
  );
}

/// A decoded `Traded` log record.
typedef DecodedTradedEvent = TradedEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
TradedEvent? parseTradedEventFromLog(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null || bytes.length < 2) {
    return null;
  }
  for (var index = 0; index < 1; index++) {
    if (bytes[index] != _tradedEventDiscriminatorBytes[index]) {
      return null;
    }
  }
  if (bytes[1] != tradedEventMigrationVersion) {
    return null;
  }
  return decodeTradedEvent(bytes);
}
