<script lang="ts">
  import type { BackupModel } from '../models/restream.model';
  import { sanitizeUrl } from '../utils/util';

  export let removeFn: () => void;
  export let onChangeFn: () => void;

  export let backup: BackupModel;

  const onIsPullChanged = (): void => {
    if (!backup.isPull) {
      backup.pullUrl = null;
    }

    onChangeFn();
  };

  const onPullUrlChanged = (): void => {
    if (backup.pullUrl !== null) {
      backup.pullUrl = sanitizeUrl(backup.pullUrl);
    }

    onChangeFn();
  };
</script>

<li class="uk-form-small uk-flex uk-flex-between backup-item">
  <span class="key-label">{backup.key}</span>
  <label>
    <input
      class="uk-checkbox"
      type="checkbox"
      bind:checked={backup.isPull}
      on:change={onIsPullChanged}
    /> pulled from</label
  >
  <input
    class="uk-input uk-form-small uk-width-expand"
    type="text"
    disabled={!backup.isPull}
    bind:value={backup.pullUrl}
    on:change={onPullUrlChanged}
    placeholder="rtmp://..."
  />
  <button class="uk-icon uk-close" on:click={removeFn} />
</li>

<style lang="stylus">
  .key-label
    width: 60px;

  .backup-item
    column-gap: 20px;

</style>
