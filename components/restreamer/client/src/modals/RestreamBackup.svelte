<script lang="ts">
  export let removeFn;
  export let key: string;
  export let url: string;
  export let isPull: boolean;
</script>
<li class='uk-form-small uk-flex uk-flex-between' style='column-gap: 20px;'>
  <span class='key-label'>{key}</span>
  <label>
    <input
      class='uk-checkbox'
      type='checkbox'
      bind:checked={isPull}
    /> pulled from</label>
  <input
    class='uk-input uk-form-small uk-width-expand'
    type='text'
    disabled={!isPull}
    bind:value={url}
    placeholder='rtmp://...'
  />
  <button class='uk-icon uk-close' uk-close on:click={removeFn}></button>
</li>
<style lang='stylus'>
  .key-label
    width: 60px;

</style>
