![alt text](../img/banner.jpg "banner")
# 5. Financial Module  <a id="financialmodule"></a>

The Oasis Financial Module is a data-driven process design for calculating the losses on (re)insurance contracts. It has an abstract design in order to cater for the many variations in contract structures and terms. The way Oasis works is to be fed data in order to execute calculations, so for the insurance calculations it needs to know the structure, parameters and calculation rules to be used. This data must be provided in the files used by the Oasis Financial Module:

* **fm_programme**: defines how coverages are grouped into accounts and programmes
* **fm_profile**: defines the layers and terms
* **fm_policytc**: defines the relationship of the contract layers
* **fm_xref**: specifies the summary level of the output losses
 
This section explains the design of the Financial Module which has been implemented in the **fmcalc** component. 
* Runtime parameters and usage instructions for fmcalc are covered in [4.1 Core Components](CoreComponents.md). 
* The formats of the input files are covered in [4.3 Data Conversion Components](DataConversionComponents.md). 
 
In addition, there is a separate github repository [ktest](https://github.com/OasisLMF/ktest) which is an extended test suite for ktools and contains a library of financial module worked examples provided by Oasis Members with a full set of input and output files.

Note that other reference tables are referred to below that do not appear explicitly in the kernel as they are not directly required for calculation.  It is expected that a front end system will hold all of the exposure and policy data and generate the above four input files required for the kernel calculation.

## Scope

The Financial Module outputs sample by sample losses by (re)insurance contract, or by item, which represents the individual coverage subject to economic loss from a particular peril. In the latter case, it is necessary to ‘back-allocate’ losses when they are calculated at a higher policy level.  The Financial Module can output retained loss or ultimate net loss (UNL) perspectives as an option, and at any stage in the calculation.

The output contains anonymous keys representing the (re)insurance policy (agg_id and layer_id) at the chosen output level (output_id) and a loss value. Losses by sample number (idx) and event (event_id) are produced.  To make sense of the output, this output must be cross-referenced with Oasis dictionaries which contain the meaningful business information.

The Financial Module does not support multi-currency calculations.

## Profiles

Profiles are used throughout the Oasis framework and are meta-data definitions with their associated data types and rules.  Profiles are used in the Financial Module to perform the elements of financial calculations used to calculate losses to (re)insurance policies.  For anything other than the most simple policy which has a blanket deductible and limit, say, a profile do not represent a policy structure on its own, but rather is to be used as a building block which can be combined with other building blocks to model a particular financial contract. In this way it is possible to model an unlimited range of structures with a limited number of profiles.

The FM Profiles form an extensible library of calculations defined within the fmcalc code that can be invoked by specifying a particular **calcrule_id** and providing the required data values such as deductible and limit, as described below.

#### Supported Profiles

See Appendix [B FM Profiles](fmprofiles.md) for more details. 

## Design

The Oasis Financial Module is a data-driven process design for calculating the losses on insurance policies. It is an abstract design in order to cater for the many variations and has four basic concepts:

1. A **programme** which defines which **items** are grouped together at which levels for the purpose of providing loss amounts to policy terms and conditions. The programme has a user-definable profile and dictionary called **prog** which holds optional reference details such as a description and account identifier. The prog table is not required for the calculation and therefore does not appear in the kernel input files.
2. A policytc **profile** which provides the parameters of the policy’s terms and conditions such as limit and deductible and calculation rules.
3. A **policytc** cross-reference file which associates a policy terms and conditions profile to each programme level aggregation.
4. A **xref** file which specifies how the output losses are summarized.

The profile not only provides the fields to be used in calculating losses (such as limit and deductible) but also which mathematical calculation (calcrule_id) to apply.

## Data requirements

The Financial Module brings together three elements in order to undertake a calculation:
* Structural information, notably which items are covered by a set of policies.
* Loss values of items.
* Policy profiles and profile values.

There are many ways an insurance loss can be calculated with many different terms and conditions. For instance, there may be deductibles applied to each element of coverage (e.g. a buildings damage deductible), some site-specific deductibles or limits, and some overall policy deductibles and limits and share. To undertake the calculation in the correct order and using the correct items (and their values) the structure and sequence of calculations must be defined. This is done in the **programme** file which defines a heirarchy of groups across a number of **levels**.  Levels drive the sequence of calculation. A financial calculation is performed at successive levels, depending on the structure of policy terms and conditions. For example there might be 3 levels representing coverage, site and policy terms and conditions. 

#### Figure 1. Example 3-level programme hierarchy
![alt text](../img/fm1.jpg "3-level programme hierarchy")

Groups are defined within levels and they represent aggregations of losses on which to perform the financial calculations.  The grouping fields are called from_agg_id and to_agg_id which represent a grouping of losses at the previous level and the present level of the hierarchy, respectively.  

Each level calculation applies to the to_agg_id groupings in the heirarchy. There is no calculation applied to the from_agg_id groupings at level 1 - these ids directly correspond to the ids in the loss input. 

#### Figure 2. Example level 1 grouping
![alt text](../img/fm2.jpg "Example level 1 grouping")

### Loss values
The initial input is the ground-up loss (GUL) table, generally coming from the main Oasis calculation of ground-up losses. Here is an example, for a two events and 1 sample (idx=1):

| event_id | item_id  | sidx   | loss   |
|:---------|----------|--------| ------:|
|       1  | 1        |    1   | 100,000|
|       1  | 2        |    1   | 10,000 |
|       1  | 3        |    1   | 2,500  |
|       1  | 4        |    1   | 400    |
|       2  | 1        |    1   | 90,000 |
|       2  | 2        |    1   | 15,000 |
|       2  | 3        |    1   | 3,000  |
|       2  | 4        |    1   | 500    |


The values represent a single ground-up loss sample for items belonging to an account. We use “programme” rather than "account" as it is more general characteristic of a client’s exposure protection needs and allows a client to have multiple programmes active for a given period.
The linkage between account and programme can be provided by a user defined **prog** dictionary, for example;

| prog_id  | account_id  | prog_name                     |
|:---------|-------------|------------------------------:|
|       1  | 1           | ABC Insurance Co. 2016 renewal|


Items 1-4 represent Structure, Other Structure, Contents and Time Element coverage ground up losses for a single property, respectively, and this example is a simple residential policy with combined property coverage terms. For this policy type, the Structure, Other Structure and Contents losses are aggregated, and a deductible and limit is applied to the total. A separate set of terms, again a simple deductible and limit, is applied to the “Time Element” coverage which, for residential policies, generally means costs for temporary accommodation. The total insured loss is the sum of the output from the combined property terms and the time element terms.

### Programme

The actual items falling into the programme are specified in the **programme** table together with the aggregation groupings that go into a given level calculation:

| from_agg_id | level_id| to_agg_id |
|:------------|---------| ---------:|
| 1           |     1   | 1         |
| 2           |     1   | 1         |
| 3           |     1   | 1         |
| 4           |     1   | 2         |
| 1           |     2   | 1         |
| 2           |     2   | 1         |

Note that from_agg_id for level_id=1 is equal to the item_id in the input loss table (but in theory from_agg_id could represent a higher level of grouping, if required). 

In level 1, items 1, 2 and 3 all have to_agg_id =1 so losses will be summed together before applying the combined deductible and limit, but item 4 (time element) will be treated separately (not aggregated) as it has to_agg_id = 2. For level 2 we have all 4 items losses (now represented by two groups from_agg_id =1 and 2 from the previous level) aggregated together as they have the same to_agg_id = 1.

### Profile

Next we have the profile description table, which list the profiles representing general policy types. Our example is represented by two general profiles which specify the input fields and mathematical operations to perform. In this example, the profile for the combined coverages and time is the same (albeit with different values) and requires a limit, a deductible, and an associated calculation rule, whereas the profile for the policy requires a limit, attachment, and share, and an associated calculation rule.

| Profile description                            | calcrule_id |
|:-----------------------------------------------|------------:|
| deductible and limit                           |   1         |
| deductible and/or attachment, limit and share  |   2         |


There is a “profile value” table for each profile containing the applicable policy terms, each identified by a policytc_id. The table below shows the list of policy terms for calcrule_id 1.

| policytc_id | deductible1 | limit1     |
|:------------|-------------| -----------|
|       1     |  1,000      | 1,000,000  |
|       2     |    2,000    | 18,000     |

And next, for calcrule_id 2, the values for the overall policy attachment, limit and share

| policytc_id | deductible1 | attachment1 | limit1       | share1     | 
|:------------|-------------|-------------| -------------|------------|
|       3     | 0           | 1,000       | 1,000,000    |    0.1     |

In practice, all profile values are stored in a single flattened format which contains all supported profile fields (see fm profile in [4.3 Data Conversion Components](DataConversionComponents.md)), but conceptually they belong in separate profile value tables.

The flattened file is;

fm_profile

| policytc_id | calcrule_id | deductible1 | deductible2 | deductible3 | attachment1 | limit1    | share1 | share2 | share3 |
|:------------|-------------|-------------| ------------|-------------|-------------|-----------|--------|--------|-------:|
|       1     |      1      | 1,000       | 0           |     0       |    0        | 1,000,000 |  0     |   0    |  0     |
|       1     |       1	    | 2,000       |     0       |    0        |    0      	|18,000     |  0     |   0    |  0     |
|       1     | 2           | 0           |     0       |    0        | 1,000       | 1,000,000 |  0.1   |   0    |  0     |

For any given profile we have one standard rule **calcrule_id**, being the mathematical function used to calculate the losses from the given profile’s fields. More information about the functions can be found in [FM Profiles](fmprofiles.md).

### Policytc
The **policytc** table specifies the (re)insurance contracts (this is a combination of agg_id and layer_id) and the separate terms and conditions which will be applied to each layer_id/agg_id for a given level. In our example, we have a limit and deductible with the same value applicable to the combination of the first three items, a limit and deductible for the fourth item (time) in level 1, and then a limit, attachment, and share applicable at level 2 covering all items. We’d represent this in terms of the distinct agg_ids as follows:

| layer_id | level_id | agg_id   | policytc_id |
|:---------|----------| ---------|------------:|
|    1     |     1    |    1     |    1        |
|    1     |     1    |    2     |    2        |
|    1     |     2    |    1     |    3        |

In words, the data in the table mean;

At Level 1;

Apply policytc_id (terms and conditions) 1 to (the sum of losses represented by) agg_id 1

Apply policytc_id 2 to agg_id 2

Then at level 2;

Apply policytc_id 3 to agg_id 1

Levels are processed in ascending order and the calculated losses from a previous level are summed according to the groupings defined in the programme table which become the input losses to the next level.

#### Layers
Layers can be used to model multiple sets of terms and conditions applied to the same losses, such as excess policies. For the lower level calculations and in the general case where there is a single contract, layer_id should be set to 1. For a given level_id and agg_id, multiple layers can be defined by setting layer_id =1,2,3 etc, and assigning a different calculation policytc_id to each.

#### Figure 3. Example of multiple layers
![alt text](../img/fm3.jpg "Example layers")

For this example at level 3, the policytc data might look as follows;

| layer_id | level_id | agg_id   | policytc_id |
|:---------|----------| ---------|------------:|
|    1     |     3    |    1     |    22       |
|    2     |     3    |    1     |    23       |


### Output and back-allocation

 
Losses are output by event, output_id and sample.  The table looks like this;

| event_id|  output_id | sidx  |  loss    |
|:--------|------------|-------|---------:|
|    1    |      1     |    1  |   455.24 |
|    2    |      1     |    1  |   345.6  |

The output_id is specified by the user in the **xref** table, and is a unique combination of agg_id and layer_id. For instance;

| output_id | agg_id   | layer_id    |
|:----------|----------|------------:|
| 1         | 1        |    1        | 
| 2         | 1        |    2        |

The output_id must be specified consistently with the back-allocation rule. Losses can either output at the contract level or back-allocated to the lowest level, which is item_id, using one of three command line options. There are three meaningful values here – don’t allocate (0) used typically for all levels where a breakdown of losses is not required in output, allocate back to items (1) in proportion to the input (ground up) losses, or allocate back to items (2) in proportion to the losses from the prior level calculation.

```
$ fmcalc -a0 # Losses are output at the contract level and not back-allocated
$ fmcalc -a1 # Losses are back-allocated to items on the basis of the input losses (e.g. ground up loss)
$ fmcalc -a2 # Losses are back-allocated to items on the basis of the prior level losses
```
The rules for specifying the output_ids in the xref table are as follows;
 
* Rule 0: there is an output_id for every agg_id and layer_id of the final level in the **policytc** table
* Rule 1: there is an output_id for every from_agg_id of the first level in the **programme** table, and for every layer_id in the final level of the **policytc** table
* Rule 2: there is an output_id for every from_agg_id of the first level in the **programme** table, and for every layer_id in the final level of the **policytc** table

To make sense of this, if there is more than one output at the contract level, then each one must be back-allocated to all of the items, with each individual loss represented by a unique output_id. 

To avoid unnecessary computation, it is recommended not to back-allocate unless losses are required to be reported at a more detailed level than the contract level (site or zip code, for example). In this case, losses are re-aggregated up from item level (represented by output_id in fmcalc output) in summarycalc, using the fmsummaryxref table.

## Reinsurance

The first run of fmcalc is designed to calculate the primary or direct insurance losses from the ground up losses of an exposure portfolio. fmcalc has been designed to be recursive, so that the 'gross' losses from the first run can be streamed back in to second and subsequent runs of fmcalc, each time with a different set of input files representing reinsurance contracts, and can output either the reinsurance gross loss, or net loss.  There are two modes of output;

* **gross** meaning the loss to the policies or reinsurance contracts, and 
* **net** being the difference between the input loss and the policy/contract losses

net loss is output when the command line parameter -n is used, otherwise output loss is gross by default.

### Supported reinsurance types

The types of reinsurance supported by the Financial Module are;
* **Facultative** proportional or excess
* **Quota share** with event limit
* **Surplus share** with event limit
* **Per risk** with event limit
* **Catastrophe excess of loss** occurrence only

### Required files

Second and subsequent runs of fmcalc require the same four fm files fm_programme, fm_policytc, fm_profile, and fm_xref.

This time, the hierarchy specified in fm_programme must be consistent with the range of output_ids from the incoming stream of losses, as specified in the fm_xref file from the previous iteration. Specifically, this means the range of values in from_agg_id at level 1 must match the range of values in output_id.

For example;

fm_xref (iteration 1)

| output_id | agg_id   | layer_id    |
|:----------|----------|------------:|
| 1         | 1        |    1        | 
| 2         | 1        |    2        |

fm_programme (iteration 2)

| from_agg_id | level_id| to_agg_id |
|:------------|---------| ---------:|
| 1           |     1   | 1         |
| 2           |     1   | 2         |
| 1           |     2   | 1         |
| 2           |     2   | 1         |

The abstraction of from_agg_id at level 1 from item_id means that losses needn't be back-allocated to item_id after every iteration of fmcalc. In fact, performance will be improved when back-allocation is minimised.

### Example - Quota share reinsurance

Using the two layer example from above, here's an example of the fm files for a simple quota share treaty with 50% ceded and 90% placed covering both policy layers.

The command to run the direct insurance followed by reinsurance might look like this;

```
$ fmcalc -p direct < guls.bin | fmcalc -p ri1 -n > ri1_net.bin
```
In this command, ground up losses are being streamed into fmcalc to calculate the insurance losses, which are streamed into fmcalc again to calculate the reinsurance net loss. The direct insurance fm files would be located in the folder 'direct' and the reinsurance fm files in the folder 'ri1'. The -n flag in the second call of fmcalc results in net losses being output to the file 'ri1_net.bin'. These are the losses to the insurer net of recoveries from the quota share treaty.

The fm_xref file from the direct insurance (first) iteration is

fm_xref

| output_id | agg_id   | layer_id    |
|:----------|----------|------------:|
| 1         | 1        |    1        | 
| 2         | 1        |    2        |


The fm files for the reinsurance (second) iteration would be as follows;

fm_programme

| from_agg_id | level_id| to_agg_id |
|:------------|---------| ---------:|
| 1           |     1   | 1         |
| 2           |     1   | 1         |

fm_policytc

| layer_id | level_id | agg_id   | policytc_id |
|:---------|----------| ---------|------------:|
|    1     |     1    |    1     |    1        |

fm_profile


| policytc_id | calcrule_id | deductible1 | deductible2 | deductible3 | attachment1 | limit1    | share1 | share2 | share3 |
|:------------|-------------|-------------| ------------|-------------|-------------|-----------|--------|--------|-------:|
|       1     |      25     | 0           | 0           |     0       |    0        | 0         |  0.5   |   0.9  |  1     |


fm_xref

| output_id | agg_id   | layer_id    |
|:----------|----------|------------:|
| 1         | 1        |    1        | 


### Inuring priority

The Financial Module can support unlimited inuring priority levels for reinsurance. Each set of contracts with equal inuring priority would be calculated in one iteration. The net losses from the first inuring priority are streamed into the second inuring priority calculation, and so on.

Where there are multiple contracts with equal inuring priority, these are implemented as layers with a single iteration.

The net calculation for iterations with multiple layers is;

net loss = max(0, input loss - layer1 loss - layer2 loss - ... - layer n loss)


[Return to top](#financialmodule)

[Go to 6. Workflows](Workflows.md)

[Back to Contents](Contents.md)
