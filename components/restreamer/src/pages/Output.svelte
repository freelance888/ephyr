<script lang="js">
  import Output from '../Output.svelte';
  import YoutubePlayer from './YoutubePlayer.svelte';

  export let state;
  export let params = {};
</script>

<template>
  {#each $state.data.allRestreams as restream}
    {#if restream.id === params.restream_id}
      {#each restream.outputs as output}
        {#if output.id === params.output_id}
          <section class="uk-section uk-section-muted single-output">
            <Output restream_id={restream.id} value={output} />
          </section>
          {#if output.previewUrl}
            <section class="uk-section uk-section-muted video-player">
              <YoutubePlayer
                restream_id={restream.id}
                preview_url={output.previewUrl}
              />
            </section>
          {/if}
        {/if}
      {/each}
    {/if}
  {/each}
</template>

<style lang="stylus">
  .single-output
    margin-top: 20px
    padding: 10px 20px 20px 20px
    max-width: 960px

    :global(.volume input)
        width: 90% !important

  .video-player
    @extend .single-output
    max-height: 800px
    min-height: 150px
</style>
