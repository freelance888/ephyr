<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { showError } from '../../utils/util';

  export let volume;
  export let restream_id;
  export let output_id;
  export let mutations;
  export let max = 200;
  export let mixin_id;

  const tuneVolumeMutation = mutation(mutations.TuneVolume);

  let level = 100;
  let muted = false;
  $: {
    // Trigger Svelte reactivity watching.
    volume.level = volume.level;
    volume.muted = volume.muted;
    // Move `volume` and `delay` to a separate function to omit triggering this
    // block when they are changed, as we're only interested in `value` changes
    // here.
    update_volumes_and_delay();
  }

  function update_volumes_and_delay() {
    level = volume.level;
    muted = volume.muted;
  }

  async function tuneVolume() {
    const variables = {
      restream_id,
      output_id,
      mixin_id: null,
      level,
      muted,
    };
    if (mixin_id) {
      variables.mixin_id = mixin_id;
    }
    try {
      await tuneVolumeMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  async function toggleVolume() {
    muted = !muted;
    await tuneVolume();
  }
</script>

<div class="uk-flex volume">
  <a href="/" on:click|preventDefault={toggleVolume}>
    {#if muted}
      <span><i class="fas fa-volume-mute" title="Muted" /></span>
    {:else}
      <span><i class="fas fa-volume-up" title="Volume" /></span>
    {/if}
  </a>
  <input
    class="uk-range"
    class:muted
    type="range"
    min="0"
    {max}
    step="1"
    bind:value={level}
    on:change={tuneVolume}
  />
  <span class="uk-margin-small-left">{muted ? 0 : level}%</span>
</div>

<style lang="stylus">
  .fa-volume-up, .fa-volume-mute
    font-size: 10px
    margin-right: 3px

  .fa-volume-mute
    color: var(--danger-color)

  .volume
    padding-left: 17px
    font-size: 10px

    .muted
      background-color: #c4c4c4
      border-radius: 8px

    a
      color: #d9d9d9
      outline: none
      &:hover
        text-decoration: none
        color: #c4c4c4

    .uk-range::-moz-range-thumb, .uk-range::-webkit-slider-thumb
      width: 7px
      height: 12px
    .uk-range
      display: inline-block
      width: 74%
      margin-top: -1px
</style>
