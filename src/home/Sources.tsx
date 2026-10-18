// The evidence surface: every quota, budget and reset channel the fourteen sources expose, split
// by whether the app reads it, could read it with the credential it already has, or would have to
// break its own rules. Reading, not claims: each row names the endpoint and the fields.
import { Icon } from '../components/icons';
import { CANDIDATES } from './candidates';
import {
  CHANNELS,
  type Channel,
  REACH_ICON,
  REACH_LABEL,
  REACH_NOTE,
  REACH_ORDER,
} from './channels';

function ChannelCard({ channel }: { channel: Channel }) {
  return (
    <li className="nx-chan" data-reach={channel.reach}>
      <details>
        <summary className="nx-chan-head">
          <strong>{channel.harness}</strong>
          <span className="nx-chan-title">{channel.title}</span>
          <span className="nx-tag" data-shape={channel.shape}>
            {channel.shape === 'window' ? 'janela' : 'fato'}
          </span>
        </summary>
        <p className="nx-chan-how">{channel.how}</p>
        {channel.caveat !== undefined && <p className="nx-chan-caveat">{channel.caveat}</p>}
        <p className="nx-meta">
          confiança {channel.confidence.toLowerCase()} · fonte: {channel.source}
        </p>
      </details>
    </li>
  );
}

export function Sources() {
  const count = (reach: Channel['reach']) =>
    CHANNELS.filter((channel) => channel.reach === reach).length;
  return (
    <div className="nx-sources">
      <section className="nx-note">
        <h3>O que ainda dá para ler</h3>
        <p>
          Levantamento de 16/10/2026 nos documentos e no código de cada ferramenta. Cada canal diz
          qual endpoint o entrega e em que forma ele entra no modelo do app: uma <b>janela</b> com
          percentual, valor e reset, ou um <b>fato</b> sem limite.
        </p>
        <p className="nx-meta">
          {count('now')} canais novos ao alcance da credencial que o app já aceita ·{' '}
          {count('already')} confirmados como já lidos · {count('forbidden')} fora das regras do
          projeto.
        </p>
      </section>
      {REACH_ORDER.map((reach) => {
        const items = CHANNELS.filter((channel) => channel.reach === reach);
        if (items.length === 0) return null;
        return (
          <details className="nx-section" key={reach} open={reach === 'now'}>
            <summary>
              <h3 className="nx-section-head">
                <Icon name={REACH_ICON[reach]} />
                {REACH_LABEL[reach]}
                <span className="nx-count">{items.length}</span>
              </h3>
            </summary>
            <p className="nx-meta nx-note-line">{REACH_NOTE[reach]}</p>
            <ul className="nx-chan-list">
              {items.map((channel) => (
                <ChannelCard key={channel.id} channel={channel} />
              ))}
            </ul>
          </details>
        );
      })}
      <details className="nx-section">
        <summary>
          <h3 className="nx-section-head">
            Fontes candidatas
            <span className="nx-count">{CANDIDATES.length}</span>
          </h3>
        </summary>
        <p className="nx-meta nx-note-line">
          Ferramentas com canal de uso oficial e uma credencial que caberia no modelo atual.
        </p>
        <ul className="nx-chan-list">
          {CANDIDATES.map((candidate) => (
            <li className="nx-chan" key={candidate.id}>
              <p className="nx-chan-head">
                <strong>{candidate.name}</strong>
                <span className="nx-tag">confiança {candidate.confidence.toLowerCase()}</span>
              </p>
              <p className="nx-chan-how">{candidate.offer}</p>
              <p className="nx-meta">credencial: {candidate.credential}</p>
            </li>
          ))}
        </ul>
      </details>
    </div>
  );
}
