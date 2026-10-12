// One renderer per ViewState; the switch is exhaustive, so a new state fails the build here.
import type { ViewContext } from '../app/windowModel';
import type { HarnessId, HarnessView } from '../lib/contract';
import { ErrorView } from './panel/ErrorView';
import { Notices } from './panel/Notices';
import { ReadyView } from './panel/ReadyView';
import { DisabledCard, NoDataCard, PanelSkeleton } from './panel/StateCards';
import { AntigravityConnect, ClaudeConnect, CodexConnect } from './setup/ConnectCards';
import { KeyConnect } from './setup/KeyConnect';
import { LitellmForm } from './setup/LitellmForm';
import { OpencodeConnect } from './setup/OpencodeConnect';

/** Consent-gated sources ask first; LiteLLM, OpenCode, Kimi and z.ai need a key before anything is read. */
function SetupSurface({ harness }: { harness: HarnessId }) {
  switch (harness) {
    case 'claude':
      return <ClaudeConnect />;
    case 'codex':
      return <CodexConnect />;
    case 'antigravity':
      return <AntigravityConnect />;
    case 'opencode':
      return <OpencodeConnect />;
    case 'kimi':
      return <KeyConnect harness="kimi" />;
    case 'zai':
      return <KeyConnect harness="zai" />;
    case 'openrouter':
      return <KeyConnect harness="openrouter" />;
    case 'deepseek':
      return <KeyConnect harness="deepseek" />;
    case 'deepinfra':
      return <KeyConnect harness="deepinfra" />;
    case 'moonshot':
      return <KeyConnect harness="moonshot" />;
    case 'novita':
      return <KeyConnect harness="novita" />;
    case 'ollama':
      return <KeyConnect harness="ollama" />;
    case 'minimax':
      return <KeyConnect harness="minimax" />;
    case 'litellm':
      return <LitellmForm />;
  }
}

export function HarnessPanel({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  switch (view.state) {
    case 'disabled':
      return <DisabledCard harness={view.id} />;
    case 'loading':
      return <PanelSkeleton />;
    case 'needs_consent':
    case 'needs_setup':
      // A gated view can still say why (a shim that went missing); the card alone would not.
      return (
        <>
          <Notices notices={view.notices} harness={view.id} placement="top" />
          <SetupSurface harness={view.id} />
          <Notices notices={view.notices} harness={view.id} placement="bottom" />
        </>
      );
    case 'no_data':
      return <NoDataCard view={view} />;
    case 'error':
      return <ErrorView view={view} ctx={ctx} />;
    case 'ready':
      return <ReadyView view={view} ctx={ctx} />;
  }
}
