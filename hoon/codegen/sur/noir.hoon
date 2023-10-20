/-  *sock
|%
::    in-progress call table entry
::
::  soot: subject knowledge
::  sake: subject battery mask
::  form: formula if known
::  root: result knowledge
::  rake: result battery mask
::  sire: @hail for call to caller, if there is one
+$  toot
  $:  soot=sock  sake=cape
      form=(unit *)  norm=(unit nomm)
      root=sock  rake=cape
      sire=(unit @hail)
  ==
::  cold state
+$  cool
  $:  core=(jug path sock)         ::  nested batteries by path
      batt=(jug ^ path)            ::  paths by outer batteries
      call=(jug [path @] [sock *]) ::  arms to exact call label
      back=(map [sock *] [path @]) ::  path/axis labels by bell
  ==
::  hint table entry
+$  hind
  $@  ~
  [%fast tire=(unit [cone=path bats=sock matt=(map @ [@hail *])])]
::    call table entry
+$  hone  [soot=sock norm=food root=sock]
::    Nomm (Nock--)
::
::  9 is rewritten to 7+2 [9 b c] -> [7 c 2 [0 1] 0 c]
::  8 is rewritten to 7+autocons+0
+$  nomm
  $%  [%par left=nomm rite=nomm]                   :: autocons
      [%one moan=*]                                :: Nock 1
      [%two cost=nomm corn=nomm rail=@hail]        :: Nock 2 - done
      [%the pell=nomm]                             :: Nock 3
      [%for mall=nomm]                             :: Nock 4
      [%ivy this=nomm that=nomm]                   :: Nock 5
      [%six what=nomm then=nomm else=nomm]         :: Nock 6
      [%eve once=nomm then=nomm]                   :: Nock 7
      [%ten here=@ twig=nomm tree=nomm]            :: Nock 10
      [%sip hint=@ then=nomm]                      :: Nock 11 (static)
      [%tip hint=@ vice=nomm then=nomm rail=@hail] :: Nock 11 (dynamic)
      [%elf rent=nomm walk=nomm]                   :: "Nock 12"
      [%not here=@]                                :: Nock 0
  ==
+$  toms
  $@  $?(%par %wot %the %for %ivy %six %eve %vee %elf)
  $%  [%two rail=@hail]
      [%ten here=@]
      [%tip hint=@ rail=@hail]
  ==
+$  food
  [=nomm ices=(map @hail [=sock form=*]) loop=(set [=sock form=*])]
--
