import { sanitizeLabel, sanitizeUrl } from '../utils/util';

export class BackupModel {
  key: string = ''
  isPull: boolean = false;
  pullUrl: string = '';
}

export class RestreamModel {
  id: string | null = null;
  key: string = '';
  label: string = '';
  isPull: boolean = false;
  pullUrl: string = '';
  withBackup: boolean = false;
  backupIsPull: boolean = false;
  backupPullUrl: string = '';
  withHls: boolean = false;

  backups: BackupModel[] = [];

  sanitizeLabel(): void {
    this.label = sanitizeLabel(this.label);
  }

  constructor(value?: any) {
    if (!value) return;

    const withHls: boolean = value.input.endpoints.some((e) => e.kind === 'HLS');

    let pullUrl: string | null = null;
    let backup: boolean | string = false;

    if (!!value.input.src && value.input.src.__typename === 'RemoteInputSrc') {
      pullUrl = value.input.src.url;
    }

    if (
      !!value.input.src &&
      value.input.src.__typename === 'FailoverInputSrc'
    ) {
      backup = true;
      if (!!value.input.src.inputs[0].src) {
        pullUrl = value.input.src.inputs[0].src.url;
      }
      if (!!value.input.src.inputs[1].src) {
        backup = value.input.src.inputs[1].src.url;
      }
    }

    this.id = value.id;
    this.key = sanitizeUrl(value.key);
    this.label = sanitizeLabel(value.label ?? '');
    this.isPull = !!pullUrl;
    this.pullUrl = sanitizeUrl(pullUrl ?? '');
    this.withBackup = !!backup;
    this.withHls = withHls;
    this.backupIsPull = typeof backup === 'string';
    this.backupPullUrl = typeof backup === 'string' ? sanitizeUrl(backup ?? '') : '';

    this.backups = [
      { key: 'Backup1', isPull: false, pullUrl: '' },
      { key: 'Backup2', isPull: false, pullUrl: '' }
    ];
  }
}
