// The real logo of each monitored tool. Sources, licences and hashes: docs/third-party-assets.md.
import type { ComponentType } from 'react';
import type { HarnessId } from '../../lib/contract';
import { AntigravityLogo } from './AntigravityLogo';
import { ClaudeLogo } from './ClaudeLogo';
import { CodexLogo } from './CodexLogo';
import { DeepinfraLogo } from './DeepinfraLogo';
import { DeepSeekLogo } from './DeepSeekLogo';
import { KimiLogo } from './KimiLogo';
import { LiteLLMLogo } from './LiteLLMLogo';
import type { LogoProps } from './LogoSvg';
import { MinimaxLogo } from './MinimaxLogo';
import { MoonshotLogo } from './MoonshotLogo';
import { NovitaLogo } from './NovitaLogo';
import { OllamaLogo } from './OllamaLogo';
import { OpenCodeLogo } from './OpenCodeLogo';
import { OpenRouterLogo } from './OpenRouterLogo';
import { ZaiLogo } from './ZaiLogo';

const LOGOS: Record<HarnessId, ComponentType<LogoProps>> = {
  claude: ClaudeLogo,
  codex: CodexLogo,
  antigravity: AntigravityLogo,
  opencode: OpenCodeLogo,
  kimi: KimiLogo,
  zai: ZaiLogo,
  openrouter: OpenRouterLogo,
  deepseek: DeepSeekLogo,
  deepinfra: DeepinfraLogo,
  moonshot: MoonshotLogo,
  novita: NovitaLogo,
  ollama: OllamaLogo,
  minimax: MinimaxLogo,
  litellm: LiteLLMLogo,
};

interface HarnessLogoProps {
  id: HarnessId;
  /** Square box in CSS pixels. */
  size?: number;
  className?: string;
}

export function HarnessLogo({ id, size = 20, className }: HarnessLogoProps) {
  const Logo = LOGOS[id];
  return <Logo size={size} className={className} />;
}
