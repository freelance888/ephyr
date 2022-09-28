<script lang="js">
  import { copyToClipboard } from '../../utils/util';

  export let url;
</script>

<template>
  <div class="url">
    <span class="url-placeholder">{url}</span>

    <button
      type="button"
      class="url-copy-btn uk-button uk-button-link uk-button-small uk-margin-small-left"
      on:click|preventDefault={() => copyToClipboard(url)}
    >
      Copy
      <i class="far fa-copy" />
    </button>
  </div>
</template>

<style lang="stylus">
  .url
    display: inline-flex
    min-width: 20em
    padding-right: 10px
    border-right: 1px dotted gray

    &:hover .url-copy-btn
      opacity: 1
      vertical-align: baseline

    .url-placeholder
      word-break: break-all

    .url-copy-btn
      align-self: flex-start
      opacity: 0
      text-transform: initial
      text-decoration: none
      font-size: 13px
      transition: 0.1s ease-in
</style>
