// A plain llama head drawn for this project in one ink. It is a neutral glyph, not Ollama Cloud's artwork:
// original/ollama.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH =
  'M12 3.2c-2.3 0-4.1 1.8-4.1 4.1 0 1 .3 1.8.9 2.5C6.6 10.7 5 13 5 15.6c0 3 2.6 5.2 7 5.2s7-2.2 7-5.2c0-2.6-1.6-4.9-3.8-5.8.6-.7.9-1.5.9-2.5 0-2.3-1.8-4.1-4.1-4.1ZM12 5.6c.9 0 1.7.8 1.7 1.7S12.9 9 12 9s-1.7-.8-1.7-1.7.8-1.7 1.7-1.7Z';

export function OllamaLogo(props: LogoProps) {
  return (
    <LogoSvg harness="ollama" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
