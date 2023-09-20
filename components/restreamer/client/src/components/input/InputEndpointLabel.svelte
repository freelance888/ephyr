<script lang="js">
  import Fa from 'svelte-fa';
  import { faEdit } from '@fortawesome/free-solid-svg-icons';
  import { faPlus } from '@fortawesome/free-solid-svg-icons';

  import { mutation } from 'svelte-apollo';

  import { SetEndpointLabel } from '../../../api/client.graphql';
  import { saveOrCloseByKeys } from '../../utils/directives.util';

  import { showError } from '../../utils/util';

  const setLabelMutation = mutation(SetEndpointLabel);

  export let endpoint;
  export let input;
  export let restream_id;
  export let show_controls;

  let label_component;
  let label_input;
  let show_edit = false;

  $: btnText = endpoint.label ? 'Edit' : 'Add label';
  $: isEditMode = !!endpoint.label;

  async function showEdit() {
    show_edit = true;
  }

  async function cancelEdit() {
    label_component.value = endpoint.label;
    show_edit = false;
  }

  async function submit() {
    const variables = {
      restream_id: restream_id,
      input_id: input.id,
      endpoint_id: endpoint.id,
      label: label_input.value,
    };
    try {
      let result_val = await setLabelMutation({ variables });
      if (result_val.data.setEndpointLabel) {
        endpoint.label = label_input.value;
        label_component.value = endpoint.label;
        show_edit = false;
      } else if (result_val.data.setEndpointLabel === null) {
        showError('No given input endpoint.');
      }
    } catch (e) {
      showError(e.message);
    }
  }

  function init_input(label_input) {
    label_input.value = endpoint.label;
    label_input.focus();
  }
</script>

<template>
  <div class="endpoint-label">
    <span
      data-testid="endpoint-label-text"
      class={endpoint.label ? 'uk-margin-small-left' : ''}
      bind:this={label_component}
      class:uk-hidden={show_edit}>{endpoint.label ? endpoint.label : ''}</span
    >
    {#if show_edit}
      <input
        type="text"
        bind:this={label_input}
        use:init_input
        use:saveOrCloseByKeys={{
          save: submit,
          close: cancelEdit,
        }}
        on:focusout|preventDefault={() => {
          cancelEdit();
        }}
      />
    {/if}
    <button
      class="edit-label-btn uk-button uk-button-link"
      class:uk-hidden={!show_controls}
      on:click|preventDefault={() => {
        showEdit();
      }}
    >
      <span class="uk-margin-small-left">{btnText}</span>
      <Fa class={!isEditMode ? 'uk-hidden' : ''} icon={faEdit} />
      <Fa class={isEditMode ? 'uk-hidden' : ''} icon={faPlus} />
    </button>
  </div>
</template>

<style lang="stylus">
  .endpoint-label
    align-items: baseline
    display: inline-flex
    color: var(--primary-text-color)

    .edit-label-btn
      color: var(--primary-text-color)
      text-transform: initial
      text-decoration: none
      font-size: 13px
      transition: 0.1s ease-in
      &:hover
        color: var(--primary-text-hover-color)
        opacity: 1
        vertical-align: baseline
</style>
