<script lang="js">
  import { sanitizeUrl } from '../utils/util';

  export let removeFn;
  export let onChangeFn;

  export let backup;

  const onIsPullChanged = () => {
    if (!backup.isPull) {
      backup.pullUrl = null;
    }

    onChangeFn();
  };

  const onPullUrlChanged = () => {
    if (backup.pullUrl !== null) {
      backup.pullUrl = sanitizeUrl(backup.pullUrl);
    }

    onChangeFn();
  };
</script>

<li class="uk-form-small uk-flex uk-flex-between uk-flex-middle backup-item">
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
  <button class="uk-icon uk-close" uk-close on:click={removeFn} />
</li>

<style lang="stylus">
  .key-label
    width: 60px;

  .backup-item
    column-gap: 20px;

</style>
