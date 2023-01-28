import { sanitizeLabel, sanitizeUrl } from '../utils/util';
import without from 'lodash/without';

export class BackupModel {
  isPull: boolean = false;
  key: string = '';
  pullUrl: string | null = null;
}

export class RestreamModel {
  readonly backupPrefix = 'backup';

  id: string | null = null;
  key: string = '';
  label: string = '';
  isPull: boolean = false;
  pullUrl: string = '';
  withHls: boolean = false;
  fileId: string = '';
  maxFilesInPlaylist: string = '';

  backups: BackupModel[] = [];

  constructor(value?: any) {
    if (!value) return;

    const withHls: boolean = value.input.endpoints.some(
      (e) => e.kind === 'HLS'
    );
    let pullUrl: string | null = null;

    if (!!value.input.src && value.input.src.__typename === 'RemoteInputSrc') {
      pullUrl = value.input.src.url;
    }

    if (
      !!value.input.src &&
      value.input.src.__typename === 'FailoverInputSrc'
    ) {
      if (!!value.input.src.inputs[0]?.src) {
        pullUrl = value.input.src.inputs[0].src.url;
      }

      const fileBackup = value.input.src.inputs?.find(
        (x) => x.key === 'file_backup'
      );
      if (fileBackup) {
        this.fileId = fileBackup.endpoints[0].fileId;
        value.input.src.inputs = without(value.input.src.inputs, fileBackup);
      }

      this.backups = value.input.src.inputs?.slice(1).map((x) => ({
        key: x.key,
        pullUrl: x.src?.url ?? null,
        isPull: !!x.src?.url,
      }));
    }

    this.id = value.id;
    this.key = sanitizeUrl(value.key);
    this.label = sanitizeLabel(value.label ?? '');
    this.isPull = !!pullUrl;
    this.pullUrl = sanitizeUrl(pullUrl ?? '');
    this.withHls = withHls;
    this.maxFilesInPlaylist = value.max_files_in_playlist ?? '';
  }

  removeBackup(index: number): void {
    this.backups.splice(index, 1);
  }

  addBackup(): void {
    const index = this.getMaxBackupIndex() + 1;
    this.backups.push({
      key: `${this.backupPrefix}${index}`,
      isPull: false,
      pullUrl: null,
    });
  }

  getMaxBackupIndex(): number {
    return this.backups
      .map((x) => Number(x.key.replace(`${this.backupPrefix}`, '')))
      .reduce((max, current) => (current > max ? current : max), 0);
  }
}
