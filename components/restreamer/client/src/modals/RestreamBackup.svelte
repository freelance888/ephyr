<script>
  export let removeFn;
  export let key;
</script>
<li class='uk-flex uk-flex-between'>
  <input
    class='uk-input uk-form-small uk-margin-small-right'
    type='text'
    bind:value={key}
    placeholder='Key'
  />
  <label class='uk-width-small uk-form-small uk-margin-small-right'>
    <input
      class='uk-checkbox'
      type='checkbox'
    /> pulled from
  </label>
  <button class='uk-icon uk-close' uk-close on:click={removeFn}></button>
</li>
