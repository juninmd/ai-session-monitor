// The UI must be able to word everything a collector can send. The inventory lives in
// src/test/vocabulary.ts; this file checks it against the Rust sources in both directions, so a
// code added over there without a sentence here (or a sentence for a code nobody sends) fails.
import { describe, expect, it } from 'vitest';
import { describeError } from '../app/errorText';
import { noticeText } from '../app/noticeText';
import { ERROR_CODES, type ErrorCode } from '../lib/contract';
import { notice, problem } from '../mock/fixtures';
import {
  COLLECTORS,
  emitters,
  factIdsOf,
  isTestFile,
  PATTERN,
  productionCode,
  rustOf,
  scan,
} from '../test/rustSources';
import {
  COPILOT_REASONS,
  CURSOR_REASONS,
  ERROR_REASONS,
  FACTS,
  FIELDS,
  LITELLM_ENDPOINT_REASONS,
  leaks,
  NOTICE_SAMPLES,
  OPENCODE_DB_REASONS,
  TITLED_IDS,
} from '../test/vocabulary';
import { hasMessage } from './index';

const reasonsOf = (code: string): string[] => [...(ERROR_REASONS[code] ?? [])].sort();
const collectorCode = COLLECTORS.map(rustOf).join('\n');
const emittedCodes = scan(collectorCode, PATTERN.noticeCode);

describe('the Rust sources the UI is checked against', () => {
  it('are found, so no check below can pass for lack of input', () => {
    for (const name of [...COLLECTORS, 'app-core']) {
      expect(rustOf(name).length, name).toBeGreaterThan(2000);
    }
  });

  it('leave inline test modules out, with Unix and Windows line ends', () => {
    for (const eol of ['\n', '\r\n']) {
      const source = ['const A: &str = "claude_a";', '#[cfg(test)]', 'mod tests {', '"claude_b" }'];
      const code = productionCode(source.join(eol));
      expect(code).toContain('claude_a');
      expect(code).not.toContain('claude_b');
    }
  });

  it.each([
    ['crates/collector-litellm/src/budget/tests.rs', true],
    ['crates/collector-opencode/src/tests/collect_flows.rs', true],
    ['crates/collector-antigravity/src/fake_language_server.rs', true],
    ['crates/collector-codex/tests/it/fixtures.rs', true],
    ['crates/collector-litellm/src/budget.rs', false],
    ['crates/collector-claude/src/protests.rs', false],
  ])('treat %s as test code: %s', (path, expected) => {
    expect(isTestFile(path)).toBe(expected);
  });
});

describe('notice codes', () => {
  it('are exactly the ones in the inventory', () => {
    expect(emittedCodes.length).toBeGreaterThanOrEqual(15);
    expect(emittedCodes).toEqual(Object.keys(NOTICE_SAMPLES).sort());
  });

  it.each(emittedCodes)('%s has a sentence', (code) => {
    expect(hasMessage(`notice.code.${code}`)).toBe(true);
  });

  it.each(Object.entries(NOTICE_SAMPLES))('%s is worded with the params it sends', (code, all) => {
    for (const params of all) {
      const text = noticeText(notice(code, 'info', params));
      expect({ params, problems: leaks(text) }).toEqual({ params, problems: [] });
    }
  });
});

describe('facts', () => {
  const rows = Object.entries(FACTS).flatMap(([harness, facts]) =>
    facts.map(([id, unit]) => ({ harness, id, unit })),
  );

  it('are listed for the sources that send any', () => {
    expect(rows.length).toBeGreaterThanOrEqual(10);
  });

  it.each(rows)('$id ($harness) has a label', ({ id }) => {
    expect(hasMessage(`fact.${id}`)).toBe(true);
  });

  it.each(Object.entries(FACTS))('%s sends exactly the ids in the inventory', (harness, facts) => {
    expect(factIdsOf(`collector-${harness}`)).toEqual(facts.map(([id]) => id).sort());
  });
});

describe('error reasons', () => {
  const reasons = Object.entries(ERROR_REASONS).flatMap(([code, list]) =>
    list.map((reason) => ({ code, reason })),
  );

  it.each(reasons)('$code / $reason is worded', ({ code, reason }) => {
    expect(hasMessage(`error.${code}.reason.${reason}.body`)).toBe(true);
  });

  it.each(reasons)('$code / $reason shows nothing raw', ({ code, reason }) => {
    const text = describeError(problem(code as ErrorCode, { reason }));
    expect(leaks(`${text.title} ${text.body}`)).toEqual([]);
  });

  it('install_failed are exactly the ones the shim code raises', () => {
    expect(scan(rustOf('collector-claude'), PATTERN.installReason)).toEqual(
      reasonsOf('install_failed'),
    );
  });

  it('tls_error are exactly the ones the LiteLLM transport raises', () => {
    expect(scan(rustOf('collector-litellm'), PATTERN.tlsReason)).toEqual(reasonsOf('tls_error'));
  });

  it('endpoint_changed are exactly the ones the LiteLLM walk raises', () => {
    expect(scan(rustOf('collector-litellm'), PATTERN.endpointReason)).toEqual(
      [...LITELLM_ENDPOINT_REASONS].sort(),
    );
  });

  it('of the Cursor store are exactly the ones it can name, under the code each becomes', () => {
    const listed = Object.values(CURSOR_REASONS).flat().sort();
    expect(scan(rustOf('collector-cursor'), PATTERN.reasonVariant)).toEqual(listed);
    for (const [code, list] of Object.entries(CURSOR_REASONS)) {
      expect(
        list.every((reason) => reasonsOf(code).includes(reason)),
        code,
      ).toBe(true);
    }
  });

  it('of the Copilot token reader are exactly the ones it can name', () => {
    expect(scan(rustOf('collector-copilot'), PATTERN.reasonVariant)).toEqual(
      Object.values(COPILOT_REASONS).flat().sort(),
    );
  });

  it('written out as literals are all in the inventory', () => {
    const known = Object.values(ERROR_REASONS).flat();
    expect(scan(collectorCode, PATTERN.reasonParam).filter((r) => !known.includes(r))).toEqual([]);
  });

  it.each(OPENCODE_DB_REASONS)('OpenCode database reason %s is worded and produced', (reason) => {
    expect(hasMessage(`notice.reason.opencode_db_unreadable.${reason}`)).toBe(true);
    expect(rustOf('collector-opencode')).toContain(`"${reason}"`);
  });
});

describe('field names', () => {
  it('are exactly the ones the Rust code rejects', () => {
    expect(scan(emitters(), PATTERN.field)).toEqual([...FIELDS].sort());
  });

  it.each(FIELDS)('%s is named, never shown as written', (field) => {
    expect(hasMessage(`field.${field}`)).toBe(true);
    const text = describeError(problem('invalid_input', { field }));
    expect(leaks(text.body)).toEqual([]);
    expect(text.body).not.toContain(field);
  });
});

describe('error params', () => {
  it.each(ERROR_CODES)('%s shows nothing raw whatever params a source adds', (code) => {
    const params = {
      host: 'gateway.exemplo.dev',
      httpStatus: '503',
      retryAfterSecs: '30',
      redirect: 'true',
      ambiguous: 'true',
      phase: 'initialize',
      rpcCode: '-32600',
      code: 'UNAVAILABLE',
      where: 'collector',
    };
    const text = describeError(problem(code, params));
    expect(leaks(`${text.title} ${text.body}`)).toEqual([]);
    expect(JSON.stringify(text)).not.toMatch(/gateway\.exemplo|initialize|32600|UNAVAILABLE/);
  });
});

describe('window ids behind the titles', () => {
  it.each(TITLED_IDS)('%s still builds %s', (crate, literal) => {
    expect(rustOf(`collector-${crate}`)).toContain(literal);
  });
});

describe('the scanners', () => {
  const FACT_FORMS =
    'Fact {\n id: "credits_balance".to_owned(), n: 1 }\n("rpm_limit", caps.rpm, "requests")';
  it.each([
    [
      'notice code',
      PATTERN.noticeCode,
      'const NO_KEY: &str = "opencode_no_key";',
      ['opencode_no_key'],
    ],
    ['notice code of another crate', PATTERN.noticeCode, 'const X: &str = "serde_json_x";', []],
    ['fact', PATTERN.fact, FACT_FORMS, ['credits_balance', 'rpm_limit']],
    ['fact with a cast', PATTERN.fact, '("expires", secs as f64, EPOCH_UNIT)', ['expires']],
    ['fact against a notice param', PATTERN.fact, '&[("reason", reason.as_str())]', []],
    ['fact against a label', PATTERN.fact, '("Opus", 1, "x")', []],
    ['install reason', PATTERN.installReason, 'failed("write_failed", true)', ['write_failed']],
    ['install reason of a call without one', PATTERN.installReason, 'view::failed(error)', []],
    ['tls reason', PATTERN.tlsReason, '_ => "ca_unreadable",', ['ca_unreadable']],
    ['endpoint reason', PATTERN.endpointReason, 'changed("no_record")', ['no_record']],
    ['literal reason', PATTERN.reasonParam, '.with_param(\n"reason",\n"no_port")', ['no_port']],
    ['field of a call', PATTERN.field, 'Err(invalid_input("shimConsent"))', ['shimConsent']],
    ['field of a param', PATTERN.field, '.with_param("field", "configDir")', ['configDir']],
  ] as const)('read %s', (_name, pattern, source, expected) => {
    expect(scan(source, pattern)).toEqual(expected);
  });
});

describe('the guard itself', () => {
  it.each([
    ['Expira em fact.expires', ['fact.expires']],
    ['notice.code.never_heard_of_it', ['notice.code.never_heard_of_it', 'never_heard_of_it']],
    ['Campo field.surprise', ['field.surprise']],
    ['A janela rolling.', []],
    ['o campo rpm_limit', ['rpm_limit']],
    ['Lido há —', ['placeholder']],
    ['Falta {name}', ['placeholder']],
    ['O settings.json não foi alterado, arquivo .pem ok.', []],
    ['Expira em 15/01/2027 08:00', []],
  ])('reads %j as %j', (text, expected) => {
    expect(leaks(text)).toEqual(expected);
  });
});
