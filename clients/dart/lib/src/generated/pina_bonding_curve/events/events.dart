// Auto-generated. Do not edit.
// ignore_for_file: type=lint

export 'event_log.dart';
export 'config_created.dart';
export 'launch_created.dart';
export 'traded.dart';
export 'completed.dart';
export 'graduated.dart';
export 'claimed.dart';

import 'event_log.dart';
import 'config_created.dart';
import 'launch_created.dart';
import 'traded.dart';
import 'completed.dart';
import 'graduated.dart';
import 'claimed.dart';

/// The program whose invocation frames emit the events decoded here.
const pinaBondingCurveEventSourceAddress =
    'CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9';

final _programInvokeLog = RegExp(r'^Program (\S+) invoke \[\d+\]$');
final _programExitLog = RegExp(r'^Program (\S+) (?:success|failed: .*)$');

/// Decode every `Program data:` line this program emitted in a transaction's
/// logs.
///
/// [logs] must be the complete, ordered log messages of one transaction. The
/// parser follows the runtime's `Program <address> invoke [n]` and
/// `Program <address> success` / `failed` frames and decodes a data line only
/// while [programAddress] is the innermost invoked program. Any program can
/// write a `Program data:` line with this program's discriminator, so data
/// lines from other programs (including ones this program invokes through CPI)
/// and lines outside any frame are skipped rather than trusted.
///
/// Unrelated lines are skipped. A line this program emitted that names an event
/// but carries a version no generated event describes throws instead of being
/// silently dropped. The per-event `parse*FromLog` helpers decode one line
/// without this attribution and are only safe for data already known to come
/// from this program.
List<PinaBondingCurveEvent> parsePinaBondingCurveEventsFromLogs(
  List<String> logs, {
  String programAddress = pinaBondingCurveEventSourceAddress,
}) {
  final discovered = <PinaBondingCurveEvent>[];
  final frames = <String>[];
  for (final log in logs) {
    final invoke = _programInvokeLog.firstMatch(log);
    if (invoke != null) {
      frames.add(invoke.group(1)!);
      continue;
    }
    if (_programExitLog.hasMatch(log)) {
      if (frames.isNotEmpty) {
        frames.removeLast();
      }
      continue;
    }
    if (frames.isEmpty || frames.last != programAddress) {
      continue;
    }
    final configCreated = parseConfigCreatedEventFromLog(log);
    if (configCreated != null) {
      discovered.add(configCreated);
      continue;
    }
    final launchCreated = parseLaunchCreatedEventFromLog(log);
    if (launchCreated != null) {
      discovered.add(launchCreated);
      continue;
    }
    final traded = parseTradedEventFromLog(log);
    if (traded != null) {
      discovered.add(traded);
      continue;
    }
    final completed = parseCompletedEventFromLog(log);
    if (completed != null) {
      discovered.add(completed);
      continue;
    }
    final graduated = parseGraduatedEventFromLog(log);
    if (graduated != null) {
      discovered.add(graduated);
      continue;
    }
    final claimed = parseClaimedEventFromLog(log);
    if (claimed != null) {
      discovered.add(claimed);
      continue;
    }
    final unknownVersion = _unrecognizedEventVersion(log);
    if (unknownVersion != null) {
      throw RangeError(unknownVersion);
    }
  }
  return discovered;
}

/// Explain a `Program data:` line that names a migration-aware event but that
/// no generated event claimed, or return null for an unrelated line.
String? _unrecognizedEventVersion(String log) {
  final bytes = decodeProgramDataLog(log);
  if (bytes == null) {
    return null;
  }
  if (bytes.length >= 1 && bytes[0] == 1) {
    return bytes.length < 2
        ? 'event "configCreated" log is too short for its version envelope'
        : 'event "configCreated" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 2) {
    return bytes.length < 2
        ? 'event "launchCreated" log is too short for its version envelope'
        : 'event "launchCreated" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 3) {
    return bytes.length < 2
        ? 'event "traded" log is too short for its version envelope'
        : 'event "traded" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 4) {
    return bytes.length < 2
        ? 'event "completed" log is too short for its version envelope'
        : 'event "completed" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 5) {
    return bytes.length < 2
        ? 'event "graduated" log is too short for its version envelope'
        : 'event "graduated" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  if (bytes.length >= 1 && bytes[0] == 6) {
    return bytes.length < 2
        ? 'event "claimed" log is too short for its version envelope'
        : 'event "claimed" log carries migration version ${bytes[1]}, which this client cannot decode; regenerate it';
  }
  return null;
}
