import { useNavigation } from '../../app/navigation';
import { noticeAction, noticeKey, noticeText, uniqueNotices } from '../../app/noticeText';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { HarnessId, Notice } from '../../lib/contract';
import { Banner, Note } from './Banner';
import { ActionError } from './ErrorActions';

function NoticeActionButton({ notice, harness }: { notice: Notice; harness: HarnessId }) {
  const action = noticeAction(notice);
  const navigation = useNavigation();
  const { installShim } = useSettingsApi();
  const task = useAsyncAction(async () => {
    if (action === 'reinstall') await installShim();
    else navigation.openSettings(harness);
  });
  if (action === null) return null;
  return (
    <>
      <div className="actions">
        <button
          type="button"
          className="btn"
          disabled={task.pending}
          onClick={() => void task.run()}
        >
          {t(`notice.action.${action}`)}
        </button>
      </div>
      {task.error && <ActionError error={task.error} harness={harness} />}
    </>
  );
}

function NoticeItem({ notice, harness }: { notice: Notice; harness: HarnessId }) {
  const text = noticeText(notice);
  if (notice.severity === 'info') {
    return (
      <>
        <Note>{text}</Note>
        <NoticeActionButton notice={notice} harness={harness} />
      </>
    );
  }
  return (
    <Banner tone={notice.severity}>
      <p>
        <span className="vh">{`${t(`notice.severity.${notice.severity}`)}: `}</span>
        {text}
      </p>
      <NoticeActionButton notice={notice} harness={harness} />
    </Banner>
  );
}

interface NoticesProps {
  notices: readonly Notice[];
  harness: HarnessId;
  /** Warnings and errors sit above the numbers; plain information sits below them. */
  placement: 'top' | 'bottom';
}

export function Notices({ notices, harness, placement }: NoticesProps) {
  const shown = uniqueNotices(notices).filter(
    (n) => (n.severity === 'info') === (placement === 'bottom'),
  );
  return (
    <>
      {shown.map((notice) => (
        <NoticeItem key={noticeKey(notice)} notice={notice} harness={harness} />
      ))}
    </>
  );
}
